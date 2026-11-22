//! Text inside a line: emphasis, links, code, mentions, icons.
//!
//! What isn't markup is text: a marker without its closing one stays as
//! it is written.

use crate::ast::{Inline, Token, TokenKind};

/// The schemes a link may have.
const SCHEMES: [&str; 13] =
    ["https", "http", "ftp", "mailto", "callto", "skype", "sips", "sip", "tg", "steam", "tel", "bitcoin", "facetime"];

/// The two-character markers and what they mark.
const DOUBLE: [(char, TokenKind); 5] = [
    ('*', TokenKind::Bold),
    ('-', TokenKind::Strike),
    ('!', TokenKind::Accent(1)),
    ('+', TokenKind::Accent(2)),
    ('?', TokenKind::Accent(3)),
];

/// The search engines' letters.
/// The search prefixes and their engines: English Wikipedia, Russian
/// Wikipedia, Google, Yandex, the site. The site has the Cyrillic ones;
/// `wr` and `y` are their Latin twins.
const SEARCH: [(&str, char); 7] = [("wr", 'в'), ("w", 'w'), ("в", 'в'), ("g", 'g'), ("я", 'я'), ("y", 'я'), ("s", 's')];

/// Parses the text of one line, `text`, starting at column `col` of line
/// `line`; its tokens go to `toks`, its plain text as `style` when given.
pub(crate) fn parse(line: usize, col: usize, text: &str, style: Option<TokenKind>, toks: &mut Vec<Token>) -> Vec<Inline> {
    let mut p = P { c: text.chars().collect(), line, col, toks };
    let n = p.c.len();
    p.run(0, n, style)
}

struct P<'t> {
    c: Vec<char>,
    line: usize,
    col: usize,
    toks: &'t mut Vec<Token>,
}

fn word_char(c: char) -> bool {
    c.is_alphanumeric()
}

impl P<'_> {
    fn tok(&mut self, from: usize, to: usize, kind: TokenKind) {
        if to > from {
            self.toks.push(Token { line: self.line, start: self.col + from, end: self.col + to, kind });
        }
    }

    fn text(&self, from: usize, to: usize) -> String {
        self.c[from..to].iter().collect()
    }

    fn is(&self, i: usize, s: &str) -> bool {
        s.chars().enumerate().all(|(k, ch)| self.c.get(i + k) == Some(&ch))
    }

    fn find(&self, from: usize, to: usize, s: &str) -> Option<usize> {
        let len = s.chars().count();
        (from..to.saturating_sub(len - 1)).find(|&k| self.is(k, s))
    }

    /// Does a word start at `i`: is the character before it not a letter
    /// or digit?
    fn boundary(&self, i: usize, from: usize) -> bool {
        i == from || !word_char(self.c[i - 1])
    }

    fn run(&mut self, from: usize, to: usize, style: Option<TokenKind>) -> Vec<Inline> {
        let mut out = Vec::new();
        let mut start = from;
        let mut i = from;
        while i < to {
            // `Word (https://...)`: the word before is the link's text.
            if self.c[i] == ' '
                && self.c.get(i + 1) == Some(&'(')
                && let Some(end) = self.paren_url(i + 1, to)
            {
                let mut w = i;
                while w > start && !self.c[w - 1].is_whitespace() {
                    w -= 1;
                }
                if w < i {
                    self.flush(&mut out, start, w, style);
                    let text = self.text(w, i);
                    self.tok(w, i, TokenKind::Link);
                    let url = self.url_parts(i + 1, end);
                    out.push(Inline::Link { text: vec![Inline::Text(text)], url });
                    i = end;
                    start = i;
                    continue;
                }
            }
            match self.at(i, from, to) {
                Some((node, next)) => {
                    self.flush(&mut out, start, i, style);
                    out.push(node);
                    i = next;
                    start = i;
                }
                None => i += 1,
            }
        }
        self.flush(&mut out, start, to, style);
        out
    }

    fn flush(&mut self, out: &mut Vec<Inline>, from: usize, to: usize, style: Option<TokenKind>) {
        if to <= from {
            return;
        }
        if let Some(k) = style {
            self.tok(from, to, k);
        }
        let s = self.text(from, to);
        match out.last_mut() {
            Some(Inline::Text(t)) => t.push_str(&s),
            _ => out.push(Inline::Text(s)),
        }
    }

    /// Where a link starting at `i` ends; `paren`: inside `( )`, which a
    /// `)` ends.
    fn url_end(&self, i: usize, to: usize, paren: bool) -> Option<usize> {
        let scheme = SCHEMES.iter().find(|s| {
            self.is(i, s) && self.c.get(i + s.len()) == Some(&':') && !(i > 0 && word_char(self.c[i - 1]))
        })?;
        let rest = i + scheme.len() + 1;
        let web = matches!(*scheme, "http" | "https" | "ftp");
        if web && !self.is(rest, "//") {
            return None;
        }
        let body = if web { rest + 2 } else { rest };
        let mut end = body;
        while end < to && !self.c[end].is_whitespace() && !(paren && self.c[end] == ')') {
            end += 1;
        }
        if !paren {
            // Punctuation after a link isn't part of it; a `)` only when
            // it closes one inside.
            while let Some(&last) = end.checked_sub(1).and_then(|e| self.c.get(e)) {
                let opens = self.c[i..end].iter().filter(|&&c| c == '(').count();
                let closes = self.c[i..end].iter().filter(|&&c| c == ')').count();
                if ".,;:!?\"'".contains(last) || (last == ')' && closes > opens) {
                    end -= 1;
                } else {
                    break;
                }
            }
        }
        (end > body).then_some(end)
    }

    /// `(url)` at `i`: where it ends, past the `)`.
    fn paren_url(&self, i: usize, to: usize) -> Option<usize> {
        if self.c.get(i) != Some(&'(') {
            return None;
        }
        let end = self.url_end(i + 1, to, true)?;
        (self.c.get(end) == Some(&')')).then_some(end + 1)
    }

    /// Tokens of `(url)` from `i` to `end` (past the `)`), and the url.
    fn url_parts(&mut self, i: usize, end: usize) -> String {
        self.tok(i, i + 1, TokenKind::Marker);
        self.tok(i + 1, end - 1, TokenKind::Url);
        self.tok(end - 1, end, TokenKind::Marker);
        self.text(i + 1, end - 1)
    }

    /// The node starting at `i`, and where it ends.
    fn at(&mut self, i: usize, from: usize, to: usize) -> Option<(Inline, usize)> {
        let c = self.c[i];
        // `{{code}}`
        if self.is(i, "{{")
            && let Some(j) = self.find(i + 2, to, "}}")
        {
            self.tok(i, i + 2, TokenKind::Marker);
            self.tok(i + 2, j, TokenKind::Code);
            self.tok(j, j + 2, TokenKind::Marker);
            return Some((Inline::Code(self.text(i + 2, j)), j + 2));
        }
        // Labels: `[[[x]]]`, `[!!x!!]`, `[++x++]`, `[??x??]`.
        for (open, close, n) in [("[[[", "]]]", 0), ("[!!", "!!]", 1), ("[++", "++]", 2), ("[??", "??]", 3)] {
            if self.is(i, open)
                && let Some(j) = self.find(i + 3, to, close)
                && j > i + 3
            {
                self.tok(i, i + 3, TokenKind::Marker);
                let inner = self.run(i + 3, j, Some(TokenKind::Label));
                self.tok(j, j + 3, TokenKind::Marker);
                return Some((Inline::Label(n, inner), j + 3));
            }
        }
        // `[text](url)`
        if c == '['
            && let Some(j) = self.find(i + 1, to, "](")
            && j > i + 1
            && let Some(end) = self.paren_url(j + 1, to)
        {
            self.tok(i, i + 1, TokenKind::Marker);
            let text = self.run(i + 1, j, Some(TokenKind::Link));
            self.tok(j, j + 1, TokenKind::Marker);
            let url = self.url_parts(j + 1, end);
            return Some((Inline::Link { text, url }, end));
        }
        // A bare link.
        if let Some(end) = self.url_end(i, to, false) {
            self.tok(i, end, TokenKind::Url);
            let url = self.text(i, end);
            return Some((Inline::Link { text: vec![Inline::Text(display(&url))], url }, end));
        }
        // `__text__`, `__text__ (url)`, `_word_`, `_word_ (url)`.
        if c == '_' {
            let found = if self.is(i, "__") { self.double(i, to, '_') } else { self.single(i, from, to, '_') };
            if let Some((inner_from, inner_to, end)) = found {
                let m = inner_from - i;
                if self.c.get(end) == Some(&' ')
                    && let Some(url_end) = self.paren_url(end + 1, to)
                {
                    self.tok(i, inner_from, TokenKind::Marker);
                    let text = self.run(inner_from, inner_to, Some(TokenKind::Link));
                    self.tok(inner_to, inner_to + m, TokenKind::Marker);
                    let url = self.url_parts(end + 1, url_end);
                    return Some((Inline::Link { text, url }, url_end));
                }
                return Some((self.styled(i, inner_from, inner_to, end, TokenKind::Italic), end));
            }
        }
        // `**x**`, `--x--`, `!!x!!`, `++x++`, `??x??`, and single ones.
        if let Some(&(_, kind)) = DOUBLE.iter().find(|(m, _)| *m == c) {
            let found = if self.c.get(i + 1) == Some(&c) { self.double(i, to, c) } else { self.single(i, from, to, c) };
            if let Some((inner_from, inner_to, end)) = found {
                return Some((self.styled(i, inner_from, inner_to, end, kind), end));
            }
        }
        // `// offtop //`, or `// offtop` to the end of the line.
        if self.is(i, "//") && (i == from || self.c[i - 1].is_whitespace()) && (i + 2 == to || self.c[i + 2] == ' ') {
            let close = (i + 2..to.saturating_sub(1)).find(|&k| {
                self.is(k, "//") && k > i + 2 && self.c[k - 1] == ' ' && (k + 2 == to || self.c[k + 2].is_whitespace())
            });
            let (inner_to, end) = close.map_or((to, to), |k| (k, k + 2));
            self.tok(i, i + 2, TokenKind::Marker);
            let inner = self.run(i + 2, inner_to, Some(TokenKind::Offtop));
            self.tok(inner_to, end, TokenKind::Marker);
            return Some((Inline::Offtop(inner), end));
        }
        // `@p/project`, `@user`, `~~user name~~`, `~user`.
        if (c == '@' || c == '~') && self.boundary(i, from) {
            if self.is(i, "~~")
                && let Some(j) = self.find(i + 2, to, "~~")
                && j > i + 2
            {
                self.tok(i, j + 2, TokenKind::Mention);
                return Some((Inline::User(self.text(i + 2, j)), j + 2));
            }
            let project = self.is(i, "@p/");
            let start = if project { i + 3 } else { i + 1 };
            let mut end = start;
            while end < to && (word_char(self.c[end]) || "_-+.".contains(self.c[end])) {
                end += 1;
            }
            while end > start && ".-".contains(self.c[end - 1]) {
                end -= 1;
            }
            if end > start {
                self.tok(i, end, TokenKind::Mention);
                let name = self.text(start, end);
                let node = if project { Inline::Project(name) } else { Inline::User(name.replace('+', " ")) };
                return Some((node, end));
            }
        }
        // `w:term`, `wr:` (`в:`), `g:`, `y:` (`я:`), `s:`.
        if self.boundary(i, from)
            && let Some(&(prefix, engine)) = SEARCH.iter().find(|(p, _)| {
                let n = p.chars().count();
                self.is(i, p) && self.c.get(i + n) == Some(&':') && self.c.get(i + n + 1).is_some_and(|c| !c.is_whitespace())
            })
        {
            let start = i + prefix.chars().count() + 1;
            let mut end = start;
            while end < to && !self.c[end].is_whitespace() {
                end += 1;
            }
            while end > start && ".,;:!?".contains(self.c[end - 1]) {
                end -= 1;
            }
            self.tok(i, end, TokenKind::Mention);
            return Some((Inline::Search(engine, self.text(start, end)), end));
        }
        // `/иконка/name/2x`, `/icon/name`.
        if c == '/' && (i == from || self.c[i - 1].is_whitespace() || !word_char(self.c[i - 1])) {
            for prefix in ["/иконка/", "/icon/"] {
                if !self.is(i, prefix) {
                    continue;
                }
                let start = i + prefix.chars().count();
                let mut end = start;
                while end < to && (self.c[end].is_ascii_alphanumeric() || self.c[end] == '-') {
                    end += 1;
                }
                if end == start {
                    continue;
                }
                let name = self.text(start, end);
                let mut size = 1;
                if self.c.get(end) == Some(&'/')
                    && let Some(d) = self.c.get(end + 1).and_then(|d| d.to_digit(10))
                    && (1..=5).contains(&d)
                    && self.c.get(end + 2) == Some(&'x')
                {
                    size = d as u8;
                    end += 3;
                }
                self.tok(i, end, TokenKind::Icon);
                return Some((Inline::Icon(name, size), end));
            }
        }
        None
    }

    /// A double marker `mm` at `i`: the content's start and end, and the
    /// end past the closing marker.
    fn double(&self, i: usize, to: usize, m: char) -> Option<(usize, usize, usize)> {
        let open = i + 2;
        if self.c.get(open).is_none_or(|c| c.is_whitespace() || *c == m) || (i > 0 && self.c[i - 1] == m) {
            return None;
        }
        let mut k = open + 1;
        while k + 2 <= to {
            if self.c[k] == m && self.c[k + 1] == m && self.c.get(k + 2) != Some(&m) {
                return Some((open, k, k + 2));
            }
            k += 1;
        }
        None
    }

    /// A single marker `m` at `i`, around a word: the content's start and
    /// end, and the end past the closing marker.
    fn single(&self, i: usize, from: usize, to: usize, m: char) -> Option<(usize, usize, usize)> {
        if !(i == from || !(word_char(self.c[i - 1]) || self.c[i - 1] == m)) {
            return None;
        }
        let open = i + 1;
        if self.c.get(open).is_none_or(|c| c.is_whitespace() || *c == m) {
            return None;
        }
        let mut k = open + 1;
        while k < to {
            let ch = self.c[k];
            if ch.is_whitespace() {
                return None;
            }
            if ch == m {
                return match self.c.get(k + 1) {
                    Some(&n) if k + 1 < to && (word_char(n) || n == m) => None,
                    _ => Some((open, k, k + 1)),
                };
            }
            k += 1;
        }
        None
    }

    fn styled(&mut self, i: usize, inner_from: usize, inner_to: usize, end: usize, kind: TokenKind) -> Inline {
        self.tok(i, inner_from, TokenKind::Marker);
        let inner = self.run(inner_from, inner_to, Some(kind));
        self.tok(inner_to, end, TokenKind::Marker);
        match kind {
            TokenKind::Bold => Inline::Bold(inner),
            TokenKind::Italic => Inline::Italic(inner),
            TokenKind::Strike => Inline::Strike(inner),
            TokenKind::Accent(n) => Inline::Accent(n, inner),
            _ => Inline::Text(String::new()),
        }
    }
}

/// How a bare link shows: without its scheme and a last `/`.
fn display(url: &str) -> String {
    let s = url.split_once("://").map_or(url, |(_, r)| r);
    s.strip_suffix('/').unwrap_or(s).to_string()
}

/// Is `text` a link and nothing else?
pub(crate) fn whole_url(text: &str) -> bool {
    let mut toks = Vec::new();
    let p = P { c: text.chars().collect(), line: 0, col: 0, toks: &mut toks };
    p.url_end(0, p.c.len(), false) == Some(p.c.len())
}

#[cfg(test)]
#[path = "inline.test.rs"]
mod tests;
