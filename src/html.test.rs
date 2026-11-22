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

#[test]
fn tables_join_cells_on_a_grid() {
    let html = render(&crate::parse("\na | < | b\n^ | ^ | c\nd | e | f\n"));
    assert!(html.contains("<tr><td colspan=\"2\" rowspan=\"2\">a</td><td>b</td></tr><tr><td>c</td></tr><tr><td>d</td><td>e</td><td>f</td></tr>"), "{html}");
    // A `^` under a header, or a `<` with nothing to join: an empty cell.
    let html = render(&crate::parse("\nh | < | x\n---\nl | ^ | r\n< | y | z\n"));
    assert!(html.contains("<tr><th colspan=\"2\">h</th><th>x</th></tr></thead><tbody><tr><td>l</td><td></td><td>r</td></tr><tr><td></td><td>y</td><td>z</td></tr>"), "{html}");
    let list = render(&crate::parse("1. a\n2) b\n"));
    assert!(list.contains("<ol class=\"t2-ol\" data-line=\"1\"><li><span class=\"t2-marker\">1.</span> a</li><li><span class=\"t2-marker\">2)</span> b</li></ol>"), "{list}");
}
