//! Blocks: what lines make, from the start of a line. A block that opens
//! (`((кат`, `>>`, a code fence) runs to its closing line; without one,
//! to the end, with an error.

use parser_rs::ParseError;

use crate::ast::*;
use crate::inline;

/// A line: its number, the column its text starts at, its text.
#[derive(Clone, Copy)]
struct L<'a> {
    n: usize,
    col: usize,
    text: &'a str,
}

impl<'a> L<'a> {
    /// The line without its first `k` characters.
    fn skip(self, k: usize) -> L<'a> {
        let at = self.text.char_indices().nth(k).map_or(self.text.len(), |(b, _)| b);
        L { n: self.n, col: self.col + k, text: &self.text[at..] }
    }

    /// The line without the whitespace it starts with.
    fn trim_start(self) -> L<'a> {
        let k = self.text.chars().take_while(|c| c.is_whitespace()).count();
        self.skip(k)
    }

    fn indent(self) -> usize {
        self.text.chars().take_while(|c| *c == ' ' || *c == '\t').count()
    }

    fn len(self) -> usize {
        self.text.chars().count()
    }
}

/// The tags of `((tag` blocks.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Tag {
    Spoiler,
    Quote,
    Center,
    Right,
    Layout,
    Column(u8),
    Code,
    Poll,
    Tile,
}

fn tag(word: &str) -> Option<Tag> {
    Some(match word.to_lowercase().as_str() {
        "кат" | "cut" | "spoiler" => Tag::Spoiler,
        "цитата" | "quote" => Tag::Quote,
        "центр" | "center" => Tag::Center,
        "вправо" | "right" => Tag::Right,
        "макет" | "layout" => Tag::Layout,
        "блок33" | "block33" => Tag::Column(33),
        "блок50" | "block50" => Tag::Column(50),
        "блок66" | "block66" => Tag::Column(66),
        "код" | "code" => Tag::Code,
        "опрос" | "poll" => Tag::Poll,
        "плитка" | "tile" => Tag::Tile,
        _ => return None,
    })
}

/// `((tag rest` on a line: the tag and the rest, trimmed.
fn opening(text: &str) -> Option<(Tag, &str)> {
    let rest = text.trim_start().strip_prefix("((")?;
    let (word, title) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
    Some((tag(word)?, title.trim()))
}

struct Parser {
    toks: Vec<Token>,
    errors: Vec<ParseError>,
}

pub(crate) fn parse(text: &str) -> Document {
    let lines: Vec<L> = text
        .split('\n')
        .enumerate()
        .map(|(i, t)| L { n: i + 1, col: 1, text: t.strip_suffix('\r').unwrap_or(t) })
        .collect();
    let mut p = Parser { toks: Vec::new(), errors: Vec::new() };
    let (blocks, _, _) = p.seq(&lines, 0, None, None);
    p.errors.sort_by_key(|e| (e.line, e.col));
    Document { blocks, errors: p.errors, tokens: p.toks }
}

const IMAGE: [&str; 7] = [".png", ".jpg", ".jpeg", ".gif", ".webp", ".bmp", ".svg"];
const VIDEO_FILES: [&str; 5] = [".webm", ".mp4", ".ogg", ".ogv", ".ogm"];
const VIDEO_SITES: [&str; 9] = [
    "youtube.com/",
    "youtu.be/",
    "vk.com/video",
    "rutube.ru/",
    "plvideo.ru/",
    "twitch.tv/",
    "vimeo.com/",
    "dailymotion.com/",
    "bilibili.com/",
];

/// What a link on a line of its own embeds, if anything.
fn media(url: &str) -> Option<Media> {
    let lower = url.to_lowercase();
    let path = lower.split(['?', '#']).next().unwrap_or("");
    if IMAGE.iter().any(|e| path.ends_with(e)) {
        Some(Media::Image)
    } else if path.ends_with(".mp3") {
        Some(Media::Audio)
    } else if VIDEO_FILES.iter().any(|e| path.ends_with(e)) || VIDEO_SITES.iter().any(|s| lower.contains(s)) {
        Some(Media::Video)
    } else if lower.contains("gist.github.com/") {
        Some(Media::Gist)
    } else {
        None
    }
}

/// A list item's marker at the start of `text` (after its indent): the
/// marker and its length with the space after it.
fn list_marker(text: &str) -> Option<(String, usize, bool)> {
    let c: Vec<char> = text.chars().collect();
    if matches!(c.first(), Some('-' | '*')) && c.get(1) == Some(&' ') && c.len() > 2 {
        return Some((c[0].to_string(), 2, false));
    }
    let mut k = 0;
    loop {
        let d = c[k..].iter().take_while(|c| c.is_ascii_digit()).count();
        if d == 0 {
            return None;
        }
        k += d;
        match c.get(k) {
            Some('.') if c.get(k + 1).is_some_and(char::is_ascii_digit) => k += 1,
            Some('.' | ')') if c.get(k + 1) == Some(&' ') && c.len() > k + 2 => {
                return Some((c[..=k].iter().collect(), k + 2, true));
            }
            _ => return None,
        }
    }
}

/// Splits a table row at its `|`s (not inside `{{ }}`), or at tabs: each
/// cell's start (in characters) and text.
fn cells(text: &str) -> Vec<(usize, &str)> {
    let tabs = !text.contains('|');
    let mut out = Vec::new();
    let (mut start_b, mut start_c) = (0, 0);
    let mut depth = 0;
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut k = 0;
    while k < chars.len() {
        let (b, ch) = chars[k];
        if text[b..].starts_with("{{") {
            depth += 1;
            k += 2;
            continue;
        }
        if depth > 0 && text[b..].starts_with("}}") {
            depth -= 1;
            k += 2;
            continue;
        }
        if depth == 0 && (ch == '|' && !tabs || ch == '\t' && tabs) {
            out.push((start_c, &text[start_b..b]));
            start_b = b + ch.len_utf8();
            start_c = k + 1;
        }
        k += 1;
    }
    out.push((start_c, &text[start_b..]));
    out
}

fn is_row(text: &str) -> bool {
    cells(text).len() > 1
}

fn is_separator(text: &str) -> bool {
    let t = text.trim();
    !t.is_empty() && t.chars().all(|c| c == '-')
}

fn number(s: &str) -> Option<f64> {
    s.trim().replace(',', ".").parse().ok()
}

fn progress(text: &str) -> Option<(Progress, usize)> {
    let t = text.trim_start();
    let inner_start = t.strip_prefix('[')?;
    let close = inner_start.find(']')?;
    let inner = inner_start[..close].trim();
    let end = text.len() - t.len() + 1 + close + 1;
    let p = if let Some(n) = inner.strip_suffix('%') {
        Progress::Percent(number(n)?)
    } else {
        let (a, b) = inner.split_once('/')?;
        let (a, b) = (a.trim(), b.trim());
        let date = |s: &str| s.len() == 10 && s.chars().filter(|c| *c == '.').count() == 2;
        if date(a) && date(b) {
            Progress::Dates(a.into(), b.into())
        } else if let Some((min, max)) = b.split_once(" - ") {
            Progress::Range(number(a)?, number(min)?, number(max)?)
        } else {
            Progress::Of(number(a)?, number(b)?)
        }
    };
    Some((p, text[..end].chars().count()))
}

impl Parser {
    fn tok(&mut self, line: usize, start: usize, end: usize, kind: TokenKind) {
        if end > start {
            self.toks.push(Token { line, start, end, kind });
        }
    }

    /// A whole line (from its text's start) as one token.
    fn line_tok(&mut self, l: L, kind: TokenKind) {
        let l = l.trim_start();
        let len = l.text.trim_end().chars().count();
        self.tok(l.n, l.col, l.col + len, kind);
    }

    fn error(&mut self, l: L, message: String) {
        let l = l.trim_start();
        self.errors.push(ParseError { message, line: l.n, col: l.col, end_col: l.col + l.len().max(1) });
    }

    fn inline(&mut self, l: L, style: Option<TokenKind>) -> Vec<Inline> {
        let l = l.trim_start();
        inline::parse(l.n, l.col, l.text.trim_end(), style, &mut self.toks)
    }

    /// Blocks from line `i` to the closing line `close` (`))`, `<<`) or
    /// the end: the blocks, the line after, whether it closed.
    fn seq(&mut self, lines: &[L], mut i: usize, close: Option<&str>, style: Option<TokenKind>) -> (Vec<Block>, usize, bool) {
        let mut out = Vec::new();
        let mut para: Vec<L> = Vec::new();
        let mut blank = true;
        let flush = |p: &mut Parser, para: &mut Vec<L>, out: &mut Vec<Block>| {
            if para.is_empty() {
                return;
            }
            let line = para[0].n;
            let text = para.iter().map(|l| p.inline(*l, style)).collect();
            out.push(Block { line, kind: BlockKind::Paragraph(text) });
            para.clear();
        };
        while i < lines.len() {
            let l = lines[i];
            let t = l.text.trim();
            if t.is_empty() {
                flush(self, &mut para, &mut out);
                blank = true;
                i += 1;
                continue;
            }
            if close == Some(t) {
                flush(self, &mut para, &mut out);
                self.line_tok(l, TokenKind::Tag);
                return (out, i + 1, true);
            }
            if let Some((block, next)) = self.block(lines, i, blank, style) {
                flush(self, &mut para, &mut out);
                out.push(block);
                i = next;
                blank = false;
                continue;
            }
            if t == "))" || t == "<<" {
                self.error(l, format!("`{t}` closes nothing: no `{}` before it", if t == "))" { "((" } else { ">>" }));
            }
            para.push(l);
            blank = false;
            i += 1;
        }
        flush(self, &mut para, &mut out);
        (out, i, false)
    }

    /// The block starting at line `i`, if one does; and the line after it.
    /// `blank`: the line before is blank (or there's none), as a table
    /// needs.
    fn block(&mut self, lines: &[L], i: usize, blank: bool, style: Option<TokenKind>) -> Option<(Block, usize)> {
        let l = lines[i];
        let t = l.text.trim_end();
        let tt = t.trim_start();
        let at = |kind| Block { line: l.n, kind };

        // Code: ```lang ... ```, "" ... "", and {{ ... }} without formatting.
        for (open, end) in [("```", "```"), ("\"\"", "\"\""), ("{{", "}}")] {
            let lang = if open == "```" { tt.strip_prefix(open) } else { (tt == open).then_some("") };
            if let Some(lang) = lang {
                if open == "```" && lang.contains('`') {
                    continue;
                }
                self.line_tok(l, TokenKind::Tag);
                let (text, next) = self.raw(lines, i + 1, end, open);
                let lang = Some(lang.trim().to_string()).filter(|s| !s.is_empty());
                let kind = if open == "{{" { BlockKind::Raw(text) } else { BlockKind::Code { lang, text } };
                return Some((at(kind), next));
            }
        }

        // ((tag ... ))
        if let Some((tag, title)) = opening(t) {
            let lt = l.trim_start();
            let tag_len = 2 + tt[2..].split(char::is_whitespace).next().unwrap_or("").chars().count();
            self.tok(l.n, lt.col, lt.col + tag_len, TokenKind::Tag);
            let title_l = lt.skip(tag_len);
            return Some(match tag {
                Tag::Code => {
                    let (text, next) = self.raw(lines, i + 1, "))", "((код");
                    let lang = Some(title.to_string()).filter(|s| !s.is_empty());
                    if lang.is_some() {
                        self.line_tok(title_l, TokenKind::Title);
                    }
                    (at(BlockKind::Code { lang, text }), next)
                }
                Tag::Poll | Tag::Tile => {
                    self.line_tok(title_l, TokenKind::Title);
                    let (body, next) = self.body(lines, i + 1);
                    let kind = if tag == Tag::Poll { self.poll(title, &body) } else { self.tile(title, &body) };
                    (at(kind), next)
                }
                _ => {
                    let inner_style = if tag == Tag::Quote { Some(TokenKind::Quote) } else { style };
                    let (blocks, next, closed) = self.seq(lines, i + 1, Some("))"), inner_style);
                    if !closed {
                        self.error(l, format!("`(({}` has no `))` closing it", &tt[2..2 + tt[2..].find(char::is_whitespace).unwrap_or(tt.len() - 2)]));
                    }
                    let kind = match tag {
                        Tag::Spoiler => {
                            let title = (!title.is_empty()).then(|| self.inline(title_l, Some(TokenKind::Title)));
                            BlockKind::Spoiler { title, blocks }
                        }
                        Tag::Quote => BlockKind::Quote(blocks),
                        Tag::Center => BlockKind::Aligned { align: Align::Center, blocks },
                        Tag::Right => BlockKind::Aligned { align: Align::Right, blocks },
                        Tag::Layout => BlockKind::Layout(blocks),
                        Tag::Column(width) => BlockKind::Column { width, blocks },
                        Tag::Code | Tag::Poll | Tag::Tile => unreachable!("read above"),
                    };
                    (at(kind), next)
                }
            });
        }

        // >> title ... <<
        if tt == ">>" || tt.starts_with(">> ") {
            let lt = l.trim_start();
            self.tok(l.n, lt.col, lt.col + 2, TokenKind::Tag);
            let title_l = lt.skip(2);
            let title = (!title_l.text.trim().is_empty()).then(|| self.inline(title_l, Some(TokenKind::Title)));
            let (blocks, next, closed) = self.seq(lines, i + 1, Some("<<"), style);
            if !closed {
                self.error(l, "`>>` has no `<<` closing it".into());
            }
            return Some((at(BlockKind::Spoiler { title, blocks }), next));
        }

        // # Heading, == Heading ==
        for mark in ['#', '='] {
            let n = tt.chars().take_while(|c| *c == mark).count();
            let rest = &tt[n..];
            let ok = n > 0 && (mark == '=' || rest.starts_with(' ')) && !rest.trim().trim_end_matches(mark).trim().is_empty();
            if ok && (mark == '#' && n <= 6 || mark == '=') {
                let lt = l.trim_start();
                self.tok(l.n, lt.col, lt.col + n, TokenKind::Marker);
                let inner = rest.trim_end_matches(mark);
                let trail = rest.chars().count() - inner.chars().count();
                let inner_l = L { n: l.n, col: lt.col + n, text: inner };
                let text = self.inline(inner_l, Some(TokenKind::Heading));
                let end = lt.col + tt.chars().count();
                self.tok(l.n, end - trail, end, TokenKind::Marker);
                return Some((at(BlockKind::Heading { level: n.min(6) as u8, text }), i + 1));
            }
        }

        // A table: rows with `|` (or tabs), after a blank line.
        if blank && is_row(t) {
            return Some(self.table(lines, i, style));
        }

        // ---
        if is_separator(t) {
            self.line_tok(l, TokenKind::Marker);
            return Some((at(BlockKind::Rule), i + 1));
        }

        // > quote
        if tt.starts_with('>') {
            let mut inner = Vec::new();
            let mut k = i;
            while k < lines.len() {
                let lk = lines[k].trim_start();
                let s = lk.text.trim_end();
                if !s.starts_with('>') || s == ">>" || s.starts_with(">> ") {
                    break;
                }
                self.tok(lk.n, lk.col, lk.col + 1, TokenKind::Marker);
                let after = lk.skip(1);
                inner.push(if after.text.starts_with(' ') { after.skip(1) } else { after });
                k += 1;
            }
            let (blocks, _, _) = self.seq(&inner, 0, None, Some(TokenKind::Quote));
            return Some((at(BlockKind::Quote(blocks)), k));
        }

        // [ 5% ] text
        if let Some((p, len)) = progress(t) {
            let lt = l.trim_start();
            self.tok(l.n, lt.col, l.col + len, TokenKind::Progress);
            let text = self.inline(l.skip(len), style);
            return Some((at(BlockKind::Progress(p, text)), i + 1));
        }

        // - list, 1. list
        if let Some((_, _, _)) = list_marker(&t[t.len() - tt.len()..]) {
            return Some(self.list(lines, i, style));
        }

        // A link on a line of its own: an image, a video ...
        let (align, url_l) = match tt.get(..3) {
            Some("<- ") => (Some(Align::Left), l.trim_start().skip(3)),
            Some("-> ") => (Some(Align::Right), l.trim_start().skip(3)),
            Some("<> ") => (Some(Align::Center), l.trim_start().skip(3)),
            _ => (None, l.trim_start()),
        };
        let url = url_l.text.trim();
        if inline::whole_url(url)
            && let Some(kind) = media(url)
        {
            if align.is_some() {
                let lt = l.trim_start();
                self.tok(l.n, lt.col, lt.col + 2, TokenKind::Marker);
            }
            self.line_tok(url_l, TokenKind::Url);
            return Some((at(BlockKind::Media { kind, url: url.to_string(), align }), i + 1));
        }
        None
    }

    /// Lines as they are from `i` to the line that is exactly `end`: the
    /// text, the line after. `open` names the block for an error.
    fn raw(&mut self, lines: &[L], i: usize, end: &str, open: &str) -> (String, usize) {
        let mut k = i;
        while k < lines.len() && lines[k].text != end {
            self.line_tok(lines[k], TokenKind::Code);
            k += 1;
        }
        let text = lines[i.min(k)..k].iter().map(|l| l.text).collect::<Vec<_>>().join("\n");
        if k < lines.len() {
            self.line_tok(lines[k], TokenKind::Tag);
            (text, k + 1)
        } else {
            self.error(lines[i - 1], format!("`{open}` has no `{end}` closing it"));
            (text, k)
        }
    }

    /// The lines of a poll or a tile, to its `))`.
    fn body<'a>(&mut self, lines: &[L<'a>], i: usize) -> (Vec<L<'a>>, usize) {
        let mut k = i;
        while k < lines.len() && lines[k].text.trim() != "))" {
            k += 1;
        }
        let body = lines[i..k].to_vec();
        if k < lines.len() {
            self.line_tok(lines[k], TokenKind::Tag);
            (body, k + 1)
        } else {
            self.error(lines[i - 1], "the block has no `))` closing it".into());
            (body, k)
        }
    }

    fn poll(&mut self, title: &str, body: &[L]) -> BlockKind {
        let mut poll = Poll { title: title.to_string(), options: Vec::new(), multiple: None, closes: None };
        for &l in body {
            let t = l.text.trim();
            if t.is_empty() {
                continue;
            }
            let (word, rest) = t.split_once(' ').unwrap_or((t, ""));
            match word.to_lowercase().as_str() {
                "мультивыбор" | "multichoice" | "multiple" => {
                    self.line_tok(l, TokenKind::Tag);
                    poll.multiple = Some(rest.trim().parse().ok());
                }
                "закрывается" | "closes" if !rest.trim().is_empty() => {
                    self.line_tok(l, TokenKind::Tag);
                    poll.closes = Some(rest.trim().to_string());
                }
                _ => poll.options.push(self.inline(l, None)),
            }
        }
        BlockKind::Poll(poll)
    }

    fn tile(&mut self, title: &str, body: &[L]) -> BlockKind {
        let mut tile = Tile { title: title.to_string(), url: String::new(), image: None, icon: None, description: None };
        let mut text = Vec::new();
        for &l in body {
            let t = l.text.trim();
            if t.is_empty() {
                continue;
            }
            if inline::whole_url(t) && tile.url.is_empty() {
                self.line_tok(l, TokenKind::Url);
                tile.url = t.to_string();
            } else if inline::whole_url(t) && media(t) == Some(Media::Image) && tile.image.is_none() {
                self.line_tok(l, TokenKind::Url);
                tile.image = Some(t.to_string());
            } else if let Some(name) = t.strip_prefix("/иконка/").or_else(|| t.strip_prefix("/icon/")) {
                self.line_tok(l, TokenKind::Icon);
                tile.icon = Some(name.to_string());
            } else {
                self.line_tok(l, TokenKind::Quote);
                text.push(t);
            }
        }
        if !text.is_empty() {
            tile.description = Some(text.join("\n"));
        }
        BlockKind::Tile(tile)
    }

    fn table(&mut self, lines: &[L], i: usize, style: Option<TokenKind>) -> (Block, usize) {
        let mut rows = Vec::new();
        let mut header = 0;
        let mut k = i;
        while k < lines.len() {
            let l = lines[k];
            let t = l.text.trim_end();
            if is_separator(t) && header == 0 && !rows.is_empty() {
                self.line_tok(l, TokenKind::Marker);
                header = rows.len();
                k += 1;
                continue;
            }
            if !is_row(t) {
                break;
            }
            let mut row = Vec::new();
            let parts = cells(t);
            for (n, (start, text)) in parts.iter().enumerate() {
                if n > 0 {
                    self.tok(l.n, l.col + start - 1, l.col + start, TokenKind::Marker);
                }
                let cl = L { n: l.n, col: l.col + start, text };
                let ct = text.trim();
                let cell = match ct {
                    "<" => Cell::Left,
                    "^" => Cell::Up,
                    _ => {
                        let cl = cl.trim_start();
                        let (align, skip) = if ct.starts_with(">>> ") {
                            (Some(Align::Right), 4)
                        } else if ct.starts_with("<> ") {
                            (Some(Align::Center), 3)
                        } else {
                            (None, 0)
                        };
                        self.tok(l.n, cl.col, cl.col + skip, TokenKind::Marker);
                        Cell::Text { align, text: self.inline(cl.skip(skip), style) }
                    }
                };
                if matches!(cell, Cell::Left | Cell::Up) {
                    let cl = cl.trim_start();
                    self.tok(l.n, cl.col, cl.col + 1, TokenKind::Marker);
                }
                row.push(cell);
            }
            rows.push(row);
            k += 1;
        }
        (Block { line: lines[i].n, kind: BlockKind::Table(Table { header, rows }) }, k)
    }

    fn list(&mut self, lines: &[L], i: usize, style: Option<TokenKind>) -> (Block, usize) {
        let base = lines[i].indent();
        let mut items: Vec<Item> = Vec::new();
        let mut ordered = false;
        let mut k = i;
        while k < lines.len() {
            let l = lines[k];
            if l.text.trim().is_empty() {
                break;
            }
            let indent = l.indent();
            let lt = l.trim_start();
            let marker = list_marker(lt.text.trim_end());
            if indent == base
                && let Some((m, len, numbered)) = marker
            {
                if items.is_empty() {
                    ordered = numbered;
                }
                self.tok(l.n, lt.col, lt.col + m.chars().count(), TokenKind::ListMarker);
                let text = self.inline(lt.skip(len), style);
                // Lines indented more, to the next item: under this one.
                let mut j = k + 1;
                while j < lines.len() && !lines[j].text.trim().is_empty() && lines[j].indent() > base {
                    j += 1;
                }
                let (blocks, _, _) = self.seq(&lines[k + 1..j], 0, None, style);
                items.push(Item { marker: m, text, blocks });
                k = j;
            } else {
                break;
            }
        }
        (Block { line: lines[i].n, kind: BlockKind::List(List { ordered, items }) }, k)
    }
}
