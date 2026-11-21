use super::*;
use Inline::*;

fn p(s: &str) -> Vec<Inline> {
    parse(1, 1, s, None, &mut Vec::new())
}

fn t(s: &str) -> Inline {
    Text(s.into())
}

#[test]
fn emphasis_double_and_single() {
    assert_eq!(p("*bold*, **two words**"), [Bold(vec![t("bold")]), t(", "), Bold(vec![t("two words")])]);
    assert_eq!(p("_it_ __it two__"), [Italic(vec![t("it")]), t(" "), Italic(vec![t("it two")])]);
    assert_eq!(p("-gone- --two gone--"), [Strike(vec![t("gone")]), t(" "), Strike(vec![t("two gone")])]);
    assert_eq!(p("!!a1!! ++a2++ ??a3??"), [Accent(1, vec![t("a1")]), t(" "), Accent(2, vec![t("a2")]), t(" "), Accent(3, vec![t("a3")])]);
    // Single ones, as the article uses them; a closing one may follow a space.
    assert_eq!(p("?byte? - 1 +0+"), [Accent(3, vec![t("byte")]), t(" - 1 "), Accent(2, vec![t("0")])]);
    assert_eq!(p("!!note: !! x"), [Accent(1, vec![t("note: ")]), t(" x")]);
    // Not markup: inside words, around spaces, unclosed.
    for s in ["C++ and a+b+c", "snake_case_name", "xgm-team", "a - b", "5 * 3", "Hello!", "**open"] {
        assert_eq!(p(s), [t(s)], "{s}");
    }
    assert_eq!(p("**bold _it_**"), [Bold(vec![t("bold "), Italic(vec![t("it")])])]);
}

#[test]
fn links() {
    let link = |text: &str, url: &str| Link { text: vec![t(text)], url: url.into() };
    assert_eq!(p("https://example.com/"), [link("example.com", "https://example.com/")]);
    assert_eq!(p("see https://x.ru/a, then"), [t("see "), link("x.ru/a", "https://x.ru/a"), t(", then")]);
    assert_eq!(p("__Link text__ (https://xgm.guru)"), [link("Link text", "https://xgm.guru")]);
    assert_eq!(p("_Word_ (https://xgm.guru)"), [link("Word", "https://xgm.guru")]);
    assert_eq!(p("a Word (https://xgm.guru) b"), [t("a "), link("Word", "https://xgm.guru"), t(" b")]);
    assert_eq!(p("[text](https://e.com)"), [link("text", "https://e.com")]);
    assert_eq!(p("{{__T__ (tg://resolve?domain=x)}}"), [Code("__T__ (tg://resolve?domain=x)".into())]);
    assert_eq!(p("__T__ (tg://resolve?domain=x)"), [link("T", "tg://resolve?domain=x")]);
}

#[test]
fn the_rest() {
    assert_eq!(p("{{no *bold*}}"), [Code("no *bold*".into())]);
    assert_eq!(p("[[[tag]]] [!!a!!]"), [Label(0, vec![t("tag")]), t(" "), Label(1, vec![t("a")])]);
    assert_eq!(p("x // aside // y"), [t("x "), Offtop(vec![t(" aside ")]), t(" y")]);
    assert_eq!(p("x // to the end"), [t("x "), Offtop(vec![t(" to the end")])]);
    assert_eq!(p("@Кет, @N7+Molot, ~~XGM Bot~~, @p/xgm"), [
        User("Кет".into()), t(", "), User("N7 Molot".into()), t(", "), User("XGM Bot".into()), t(", "), Project("xgm".into()),
    ]);
    assert_eq!(p("mail a@b.c"), [t("mail a@b.c")]);
    assert_eq!(p("w:English_Wikipedia, я:Яндекс"), [Search('w', "English_Wikipedia".into()), t(", "), Search('я', "Яндекс".into())]);
    assert_eq!(p("/иконка/burger/3x /icon/cube"), [Icon("burger".into(), 3), t(" "), Icon("cube".into(), 1)]);
}

#[test]
fn tokens_cover_markup() {
    let mut toks = Vec::new();
    parse(4, 3, "**b** {{c}}", None, &mut toks);
    let kinds: Vec<(usize, usize, TokenKind)> = toks.iter().map(|t| (t.start, t.end, t.kind)).collect();
    assert_eq!(kinds, [
        (3, 5, TokenKind::Marker), (5, 6, TokenKind::Bold), (6, 8, TokenKind::Marker),
        (9, 11, TokenKind::Marker), (11, 12, TokenKind::Code), (12, 14, TokenKind::Marker),
    ]);
}
