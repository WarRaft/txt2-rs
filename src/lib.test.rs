use super::*;

fn kinds(text: &str) -> Vec<BlockKind> {
    parse(text).blocks.into_iter().map(|b| b.kind).collect()
}

fn t(s: &str) -> Inline {
    Inline::Text(s.into())
}

#[test]
fn headings_rules_paragraphs() {
    let doc = parse("# One\n=== Two ===\n==Three\n\nline one\nline two\n---\n");
    assert_eq!(doc.blocks.iter().map(|b| b.line).collect::<Vec<_>>(), [1, 2, 3, 5, 7]);
    assert_eq!(doc.blocks[0].kind, BlockKind::Heading { level: 1, text: vec![t("One")] });
    assert_eq!(doc.blocks[1].kind, BlockKind::Heading { level: 3, text: vec![t("Two")] });
    assert_eq!(doc.blocks[2].kind, BlockKind::Heading { level: 2, text: vec![t("Three")] });
    assert_eq!(doc.blocks[3].kind, BlockKind::Paragraph(vec![vec![t("line one")], vec![t("line two")]]));
    assert_eq!(doc.blocks[4].kind, BlockKind::Rule);
    assert!(doc.errors.is_empty());
}

#[test]
fn blocks_that_close() {
    let k = kinds("((кат Title\ninside\n))\n>> \ncut\n<<\n((цитата\nq\n))\n((центр\nc\n))\n((макет\n((блок33\na\n))\n((block66\nb\n))\n))\n");
    assert!(matches!(&k[0], BlockKind::Spoiler { title: Some(t), blocks } if t == &vec![Inline::Text("Title".into())] && blocks.len() == 1));
    assert!(matches!(&k[1], BlockKind::Spoiler { title: None, .. }));
    assert!(matches!(&k[2], BlockKind::Quote(b) if b.len() == 1));
    assert!(matches!(&k[3], BlockKind::Aligned { align: Align::Center, .. }));
    let BlockKind::Layout(cols) = &k[4] else { panic!("{:?}", k[4]) };
    assert!(matches!(cols[0].kind, BlockKind::Column { width: 33, .. }));
    assert!(matches!(cols[1].kind, BlockKind::Column { width: 66, .. }));

    let doc = parse("((кат open\ntext\n");
    assert_eq!(doc.errors.len(), 1);
    assert_eq!((doc.errors[0].line, doc.errors[0].message.as_str()), (1, "`((кат` has no `))` closing it"));
    assert_eq!(parse("))\n").errors[0].message, "`))` closes nothing: no `((` before it");
}

#[test]
fn code_and_raw() {
    let k = kinds("```cpp\nint *a*;\n```\n\"\"\nx\n\"\"\n((код jass\ncall X()\n))\n{{\n> *raw*\n}}\n```\n``` \n```\n");
    assert_eq!(k[0], BlockKind::Code { lang: Some("cpp".into()), text: "int *a*;".into() });
    assert_eq!(k[1], BlockKind::Code { lang: None, text: "x".into() });
    assert_eq!(k[2], BlockKind::Code { lang: Some("jass".into()), text: "call X()".into() });
    assert_eq!(k[3], BlockKind::Raw("> *raw*".into()));
    // A fence with a space after it is content.
    assert_eq!(k[4], BlockKind::Code { lang: None, text: "``` ".into() });
}

#[test]
fn quotes_and_lists() {
    let k = kinds("> one\n>\n> > two\n\n- a\n- b\n 1. c\n   * d\n 2) e\n  2.1. f\n- g\n");
    let BlockKind::Quote(q) = &k[0] else { panic!() };
    assert!(matches!(q.last().map(|b| &b.kind), Some(BlockKind::Quote(_))));
    let BlockKind::List(l) = &k[1] else { panic!("{:?}", k[1]) };
    assert!(!l.ordered);
    assert_eq!(l.items.iter().map(|i| i.marker.as_str()).collect::<Vec<_>>(), ["-", "-", "-"]);
    let BlockKind::List(sub) = &l.items[1].blocks[0].kind else { panic!() };
    assert!(sub.ordered);
    assert_eq!(sub.items.iter().map(|i| i.marker.as_str()).collect::<Vec<_>>(), ["1.", "2)"]);
    assert!(matches!(&sub.items[0].blocks[0].kind, BlockKind::List(d) if !d.ordered));
    assert!(matches!(&sub.items[1].blocks[0].kind, BlockKind::List(f) if f.items[0].marker == "2.1."));
}

#[test]
fn tables() {
    let k = kinds("\nhead | < | c\n---\n<> a | >>> b | ^\n^ | x {{|}} y |\n\nno|table\n");
    let BlockKind::Table(t) = &k[0] else { panic!("{:?}", k[0]) };
    assert_eq!(t.header, 1);
    assert_eq!(t.rows[0][1], Cell::Left);
    assert!(matches!(&t.rows[1][0], Cell::Text { align: Some(Align::Center), .. }));
    assert!(matches!(&t.rows[1][1], Cell::Text { align: Some(Align::Right), .. }));
    assert_eq!(t.rows[2][0], Cell::Up);
    assert_eq!(t.rows[2].len(), 3);
    assert!(matches!(&t.rows[2][1], Cell::Text { text, .. } if text.contains(&Inline::Code("|".into()))));
    // After a blank line only; `no|table` is the start of one.
    assert!(matches!(&k[1], BlockKind::Table(_)));
    assert!(matches!(&kinds("text\na|b\n")[0], BlockKind::Paragraph(_)));
    // Tabs, pasted from a spreadsheet.
    assert!(matches!(&kinds("a\tb\nc\td\n")[0], BlockKind::Table(t) if t.rows.len() == 2));
}

#[test]
fn media_progress_polls_tiles() {
    let k = kinds("https://x.ru/a.png\n<- https://x.ru/b.jpg?x=1\nhttps://youtu.be/abc\nhttps://x.ru/s.mp3\nhttps://gist.github.com/a/1\n[ 5% ]\n[ 2 / 5 ] two of five\n[ 4 / 3 - 19 ]\n[ 28.11.2023 / 27.11.2025 ]\n");
    assert_eq!(k[0], BlockKind::Media { kind: Media::Image, url: "https://x.ru/a.png".into(), align: None });
    assert_eq!(k[1], BlockKind::Media { kind: Media::Image, url: "https://x.ru/b.jpg?x=1".into(), align: Some(Align::Left) });
    assert!(matches!(k[2], BlockKind::Media { kind: Media::Video, .. }));
    assert!(matches!(k[3], BlockKind::Media { kind: Media::Audio, .. }));
    assert!(matches!(k[4], BlockKind::Media { kind: Media::Gist, .. }));
    assert_eq!(k[5], BlockKind::Progress(Progress::Percent(5.0), vec![]));
    assert_eq!(k[6], BlockKind::Progress(Progress::Of(2.0, 5.0), vec![t("two of five")]));
    assert_eq!(k[7], BlockKind::Progress(Progress::Range(4.0, 3.0, 19.0), vec![]));
    assert_eq!(k[8], BlockKind::Progress(Progress::Dates("28.11.2023".into(), "27.11.2025".into()), vec![]));

    let k = kinds("((опрос Цвет?\nмультивыбор 2\nзакрывается 15.09.2028\nКрасный\nСиний\n))\n((плитка Мои\nhttps://xgm.guru/p/a\nhttps://x.ru/i.png\nОписание\n))\n");
    let BlockKind::Poll(p) = &k[0] else { panic!() };
    assert_eq!((p.title.as_str(), p.options.len(), p.multiple, p.closes.as_deref()), ("Цвет?", 2, Some(Some(2)), Some("15.09.2028")));
    let BlockKind::Tile(tile) = &k[1] else { panic!() };
    assert_eq!(
        (tile.url.as_str(), tile.image.as_deref(), tile.description.as_deref()),
        ("https://xgm.guru/p/a", Some("https://x.ru/i.png"), Some("Описание"))
    );
}

#[test]
fn html_and_tokens() {
    let doc = parse("== T ==\n\n?int?|count|+0+\n|i times:|\n");
    let html = html::render(&doc);
    assert!(html.contains("<h2 data-line=\"1\">T</h2>"), "{html}");
    assert!(html.contains("<table data-line=\"3\"><tr><td><span class=\"t2-accent3\">int</span></td><td>count</td>"), "{html}");
    assert!(html.contains("<tr><td></td><td>i times:</td><td></td></tr>"), "{html}");
    let merged = html::render(&parse("\na | < | b\n^ | ^ | c\n"));
    assert!(merged.contains("<td colspan=\"2\" rowspan=\"2\">a</td><td>b</td></tr><tr><td>c</td>"), "{merged}");
    let encoded = semantic::encoded_semantic_tokens("**b** x\n");
    assert_eq!(encoded.len(), 15);
}

/// The documentation's pages (copied from the site, with its noise) all
/// parse and render, and their tokens are in their lines.
#[test]
fn the_documentation_pages() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("test");
    let mut n = 0;
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "txt2") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        let doc = parse(&text);
        assert!(!html::render(&doc).is_empty());
        let lines: Vec<usize> = text.split('\n').map(|l| l.chars().count()).collect();
        for t in &doc.tokens {
            assert!(t.start < t.end && t.end <= lines[t.line - 1] + 1, "{}: {t:?}", path.display());
        }
        n += 1;
    }
    assert!(n >= 10);
}
