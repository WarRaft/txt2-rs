use super::*;

#[test]
fn escapes_and_spoilers() {
    let html = render(&crate::parse(">>\n<b> & \"x\"\n<<\n"));
    assert_eq!(html, "<details class=\"t2-spoiler\" data-line=\"1\"><summary>Show</summary><div class=\"t2-spoiler-body\"><p data-line=\"2\">&lt;b&gt; &amp; &quot;x&quot;</p>\n</div></details>\n");
    assert!(page(&crate::parse("x"), "a<b").contains("<title>a&lt;b</title>"));
}

#[test]
fn icons_are_fontawesome() {
    let html = render(&crate::parse("/иконка/burger/3x /icon/cube\n"));
    assert!(html.contains("<i class=\"fa fa-solid fa-burger fa-3x\" title=\"burger\"></i> <i class=\"fa fa-solid fa-cube\" title=\"cube\"></i>"), "{html}");
    let tile = render(&crate::parse("((плитка T\nhttps://x.ru\n/иконка/dice-d6\n))\n"));
    assert!(tile.contains("<span class=\"t2-tile-icon\"><i class=\"fa fa-solid fa-dice-d6 fa-2x\""), "{tile}");
    assert!(page(&crate::parse(""), "t").contains(FONTAWESOME));
}
