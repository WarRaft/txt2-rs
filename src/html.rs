//! HTML for a preview: a fragment, each block with `data-line` (its
//! first line), and classes the preview's stylesheet styles (`t2-`).

use crate::ast::*;

fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out += "&amp;",
            '<' => out += "&lt;",
            '>' => out += "&gt;",
            '"' => out += "&quot;",
            c => out.push(c),
        }
    }
    out
}

/// The preview's stylesheet: VS Code's theme colors where it shows the
/// page (`--vscode-*`), plain ones elsewhere.
pub const STYLE: &str = include_str!("style.css");

/// FontAwesome 6, the icons' font, for [`page`]: the version the site's
/// documentation names. An editor's preview brings its own.
pub const FONTAWESOME: &str = "https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.1.2/css/all.min.css";

/// A whole page: [`STYLE`] and the document, for a browser.
pub fn page(doc: &Document, title: &str) -> String {
    format!(
        "<!doctype html>\n<html><head><meta charset=\"utf-8\"><title>{}</title>\n<link rel=\"stylesheet\" href=\"{FONTAWESOME}\">\n<style>{STYLE}</style></head>\n<body class=\"t2\">\n{}</body></html>\n",
        esc(title),
        render(doc)
    )
}

/// The HTML of a document's blocks.
pub fn render(doc: &Document) -> String {
    let mut out = String::new();
    blocks(&mut out, &doc.blocks);
    out
}

fn blocks(out: &mut String, list: &[Block]) {
    for b in list {
        block(out, b);
    }
}

fn align_class(a: Align) -> &'static str {
    match a {
        Align::Left => "left",
        Align::Center => "center",
        Align::Right => "right",
    }
}

fn block(out: &mut String, b: &Block) {
    let line = b.line;
    match &b.kind {
        BlockKind::Paragraph(lines) => {
            *out += &format!("<p data-line=\"{line}\">");
            for (n, l) in lines.iter().enumerate() {
                if n > 0 {
                    *out += "<br>";
                }
                inlines(out, l);
            }
            *out += "</p>\n";
        }
        BlockKind::Heading { level, text } => {
            let h = (*level).clamp(1, 6);
            *out += &format!("<h{h} data-line=\"{line}\">");
            inlines(out, text);
            *out += &format!("</h{h}>\n");
        }
        BlockKind::Rule => *out += &format!("<hr data-line=\"{line}\">\n"),
        BlockKind::Quote(inner) => {
            *out += &format!("<blockquote data-line=\"{line}\">");
            blocks(out, inner);
            *out += "</blockquote>\n";
        }
        BlockKind::Code { lang, text } => {
            let class = lang.as_deref().map(|l| format!(" class=\"lang-{}\"", esc(l))).unwrap_or_default();
            *out += &format!("<pre data-line=\"{line}\"><code{class}>{}</code></pre>\n", esc(text));
        }
        BlockKind::Raw(text) => *out += &format!("<div class=\"t2-raw\" data-line=\"{line}\">{}</div>\n", esc(text).replace('\n', "<br>")),
        BlockKind::Spoiler { title, blocks: inner } => {
            *out += &format!("<details class=\"t2-spoiler\" data-line=\"{line}\"><summary>");
            match title {
                Some(t) => inlines(out, t),
                None => *out += "Show",
            }
            *out += "</summary><div class=\"t2-spoiler-body\">";
            blocks(out, inner);
            *out += "</div></details>\n";
        }
        BlockKind::Aligned { align, blocks: inner } => {
            *out += &format!("<div class=\"t2-{}\" data-line=\"{line}\">", align_class(*align));
            blocks(out, inner);
            *out += "</div>\n";
        }
        BlockKind::Layout(inner) => {
            *out += &format!("<div class=\"t2-layout\" data-line=\"{line}\">");
            blocks(out, inner);
            *out += "</div>\n";
        }
        BlockKind::Column { width, blocks: inner } => {
            *out += &format!("<div class=\"t2-column t2-w{width}\" data-line=\"{line}\">");
            blocks(out, inner);
            *out += "</div>\n";
        }
        BlockKind::List(l) => list(out, l, Some(line)),
        BlockKind::Table(t) => table(out, t, line),
        BlockKind::Progress(p, text) => {
            let (percent, label) = match p {
                Progress::Percent(v) => (*v, format!("{v}%")),
                Progress::Of(a, b) => {
                    let v = if *b != 0.0 { a / b * 100.0 } else { 0.0 };
                    (v, format!("{a} / {b} ({}%)", v.round()))
                }
                Progress::Range(v, min, max) => {
                    let p = if max > min { (v - min) / (max - min) * 100.0 } else { 0.0 };
                    (p, format!("{}%", p.round()))
                }
                Progress::Dates(from, to) => (0.0, format!("{} → {}", esc(from), esc(to))),
            };
            let percent = percent.clamp(0.0, 100.0);
            *out += &format!(
                "<div class=\"t2-progress-line\" data-line=\"{line}\"><span class=\"t2-progress\"><span style=\"width:{percent:.1}%\"></span><em>{label}</em></span> "
            );
            inlines(out, text);
            *out += "</div>\n";
        }
        BlockKind::Media(e) => {
            let a = e.align.map_or("center", align_class);
            *out += &format!("<div class=\"t2-media t2-{a}\" data-line=\"{line}\">");
            embed(out, e);
            *out += "</div>\n";
        }
        BlockKind::Gallery(list) => {
            *out += &format!("<div class=\"t2-gallery\" data-line=\"{line}\">");
            for e in list {
                embed(out, e);
            }
            *out += "</div>\n";
        }
        BlockKind::Poll(p) => {
            *out += &format!("<div class=\"t2-poll\" data-line=\"{line}\"><div class=\"t2-poll-title\">Poll: {}</div><ol>", esc(&p.title));
            for o in &p.options {
                *out += "<li>";
                inlines(out, o);
                *out += "</li>";
            }
            *out += "</ol>";
            if let Some(n) = p.multiple {
                let n = n.unwrap_or((p.options.len() / 2) as u32);
                *out += &format!("<div class=\"t2-note\">Multiple choice: choose {n}</div>");
            }
            if let Some(c) = &p.closes {
                *out += &format!("<div class=\"t2-note\">Closes {} at 12:00 Moscow time</div>", esc(c));
            }
            *out += "</div>\n";
        }
        BlockKind::Tile(t) => {
            *out += &format!("<a class=\"t2-tile\" data-line=\"{line}\" href=\"{}\">", esc(&t.url));
            if let Some(img) = &t.image {
                *out += &format!("<img src=\"{}\" alt=\"\">", esc(img));
            } else if let Some(icon) = &t.icon {
                let (name, size) = icon.split_once('/').map_or((icon.as_str(), 2), |(n, s)| {
                    (n, s.trim_end_matches('x').parse().unwrap_or(2))
                });
                *out += &format!("<span class=\"t2-tile-icon\">{}</span>", self::icon(name, size));
            }
            *out += &format!("<strong>{}</strong>", esc(&t.title));
            if let Some(d) = &t.description {
                *out += &format!("<small>{}</small>", esc(d));
            }
            *out += "</a>\n";
        }
    }
}

/// What a link on its own line shows, with its caption.
fn embed(out: &mut String, e: &Embed) {
    let u = esc(&e.url);
    let inner = match e.kind {
        Media::Image => {
            let href = esc(e.link.as_deref().unwrap_or(&e.url));
            format!("<a href=\"{href}\"><img src=\"{u}\" alt=\"\"></a>")
        }
        Media::Audio => format!("<audio controls src=\"{u}\"></audio>"),
        Media::Video if [".webm", ".mp4", ".ogg", ".ogv", ".ogm"].iter().any(|x| e.url.to_lowercase().ends_with(x)) => {
            format!("<video controls src=\"{u}\"></video>")
        }
        Media::Video => format!("<a class=\"t2-embed\" href=\"{u}\">▶ {u}</a>"),
        Media::Gist => format!("<a class=\"t2-embed\" href=\"{u}\">{{ }} gist: {u}</a>"),
    };
    *out += "<figure>";
    *out += &inner;
    if let Some(c) = &e.caption {
        *out += "<figcaption>";
        inlines(out, c);
        *out += "</figcaption>";
    }
    *out += "</figure>";
}

/// A FontAwesome 6 icon (its solid style), `size` 1 to 5 times the
/// text's.
fn icon(name: &str, size: u8) -> String {
    let size = if size > 1 { format!(" fa-{}x", size.min(5)) } else { String::new() };
    format!("<i class=\"fa fa-solid fa-{}{size}\" title=\"{}\"></i>", esc(name), esc(name))
}

fn list(out: &mut String, l: &List, line: Option<usize>) {
    let tag = if l.ordered { "ol" } else { "ul" };
    let at = line.map(|n| format!(" data-line=\"{n}\"")).unwrap_or_default();
    *out += &format!("<{tag}{at}>");
    for item in &l.items {
        *out += "<li>";
        if l.ordered && item.marker.trim_end_matches(['.', ')']).contains('.') {
            *out += &format!("<span class=\"t2-marker\">{}</span> ", esc(&item.marker));
        }
        inlines(out, &item.text);
        blocks(out, &item.blocks);
        *out += "</li>";
    }
    *out += &format!("</{tag}>\n");
}

fn table(out: &mut String, t: &Table, line: usize) {
    let width = t.rows.iter().map(Vec::len).max().unwrap_or(0);
    // Each cell's span: columns to its right joined to it (`<`), rows
    // below (`^`).
    let get = |r: usize, c: usize| t.rows.get(r).and_then(|row| row.get(c));
    *out += &format!("<table data-line=\"{line}\">");
    for r in 0..t.rows.len() {
        if r == 0 && t.header > 0 {
            *out += "<thead>";
        }
        if r == t.header && t.header > 0 {
            *out += "<tbody>";
        }
        *out += "<tr>";
        let cell_tag = if r < t.header { "th" } else { "td" };
        for c in 0..width {
            match get(r, c) {
                Some(Cell::Left | Cell::Up) => continue,
                cell => {
                    let mut cols = 1;
                    while matches!(get(r, c + cols), Some(Cell::Left)) {
                        cols += 1;
                    }
                    let mut rows = 1;
                    while matches!(get(r + rows, c), Some(Cell::Up)) {
                        rows += 1;
                    }
                    let mut attrs = String::new();
                    if cols > 1 {
                        attrs += &format!(" colspan=\"{cols}\"");
                    }
                    if rows > 1 {
                        attrs += &format!(" rowspan=\"{rows}\"");
                    }
                    if let Some(Cell::Text { align: Some(a), .. }) = cell {
                        attrs += &format!(" class=\"t2-{}\"", align_class(*a));
                    }
                    *out += &format!("<{cell_tag}{attrs}>");
                    if let Some(Cell::Text { text, .. }) = cell {
                        inlines(out, text);
                    }
                    *out += &format!("</{cell_tag}>");
                }
            }
        }
        *out += "</tr>";
        if r + 1 == t.header {
            *out += "</thead>";
        }
    }
    if t.header > 0 && t.header < t.rows.len() {
        *out += "</tbody>";
    }
    *out += "</table>\n";
}

fn inlines(out: &mut String, list: &[Inline]) {
    for i in list {
        inline(out, i);
    }
}

fn wrap(out: &mut String, open: &str, close: &str, inner: &[Inline]) {
    *out += open;
    inlines(out, inner);
    *out += close;
}

fn inline(out: &mut String, i: &Inline) {
    match i {
        Inline::Text(t) => *out += &esc(t),
        Inline::Bold(x) => wrap(out, "<b>", "</b>", x),
        Inline::Italic(x) => wrap(out, "<i>", "</i>", x),
        Inline::Strike(x) => wrap(out, "<s>", "</s>", x),
        Inline::Accent(n, x) => wrap(out, &format!("<span class=\"t2-accent{n}\">"), "</span>", x),
        Inline::Offtop(x) => wrap(out, "<span class=\"t2-offtop\">", "</span>", x),
        Inline::Label(n, x) => wrap(out, &format!("<span class=\"t2-label t2-label{n}\">"), "</span>", x),
        Inline::Code(c) => *out += &format!("<code>{}</code>", esc(c)),
        Inline::Link { text, url } => wrap(out, &format!("<a href=\"{}\">", esc(url)), "</a>", text),
        Inline::User(name) => {
            *out += &format!("<a class=\"t2-user\" href=\"https://xgm.guru/user/{}\">{}</a>", esc(&name.replace(' ', "+")), esc(name));
        }
        Inline::Project(name) => *out += &format!("<a class=\"t2-project\" href=\"https://xgm.guru/p/{0}\">{0}</a>", esc(name)),
        Inline::Search(engine, term) => {
            let q = esc(term);
            let url = match engine {
                'w' => format!("https://en.wikipedia.org/wiki/{q}"),
                'в' => format!("https://ru.wikipedia.org/wiki/{q}"),
                'g' => format!("https://www.google.com/search?q={q}"),
                'я' => format!("https://yandex.ru/search/?text={q}"),
                _ => format!("https://xgm.guru/search?text={q}"),
            };
            *out += &format!("<a class=\"t2-search\" href=\"{url}\">{}</a>", esc(&term.replace('+', " ")));
        }
        Inline::Icon(name, size) => *out += &icon(name, *size),
    }
}

#[cfg(test)]
#[path = "html.test.rs"]
mod tests;
