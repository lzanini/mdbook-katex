use super::*;

#[test]
fn test_katex_rendering_vmatrix() {
    let math_expr = r"\begin{vmatrix}a&b\\c&d\end{vmatrix}";
    let cfg = KatexConfig::default();
    let (_, display_opts, _) = cfg.build_opts_from_macros(HashMap::new());
    let _ = katex::render_with_opts(math_expr, display_opts).unwrap();
}

#[test]
fn test_rendering_vmatrix() {
    let raw_content = r"$$\begin{vmatrix}a&b\\c&d\end{vmatrix}$$";
    let (stylesheet_header, rendered_content) = test_render(raw_content);
    debug_assert_eq!(
        stylesheet_header+
        "<span class=\"katex-display\"><span class=\"katex\"><span class=\"katex-html\" aria-hidden=\"true\"><span class=\"base\"><span class=\"strut\" style=\"height:2.4em;vertical-align:-0.95em;\"></span><span class=\"minner\"><span class=\"mopen\"><span class=\"delimsizing mult\"><span class=\"vlist-t vlist-t2\"><span class=\"vlist-r\"><span class=\"vlist\" style=\"height:1.45em;\"><span style=\"top:-3.45em;\"><span class=\"pstrut\" style=\"height:4.4em;\"></span><span style=\"width:0.333em;height:2.400em;\"><svg xmlns=\"http://www.w3.org/2000/svg\" width='0.333em' height='2.400em' viewBox='0 0 333 2400'><path d='M145 15 v585 v1200 v585 c2.667,10,9.667,15,21,15 c10,0,16.667,-5,20,-15 v-585 v-1200 v-585 c-2.667,-10,-9.667,-15,-21,-15 c-10,0,-16.667,5,-20,15z M188 15 H145 v585 v1200 v585 h43z'/></svg></span></span></span><span class=\"vlist-s\">\u{200b}</span></span><span class=\"vlist-r\"><span class=\"vlist\" style=\"height:0.95em;\"><span></span></span></span></span></span></span><span class=\"mord\"><span class=\"mtable\"><span class=\"col-align-c\"><span class=\"vlist-t vlist-t2\"><span class=\"vlist-r\"><span class=\"vlist\" style=\"height:1.45em;\"><span style=\"top:-3.61em;\"><span class=\"pstrut\" style=\"height:3em;\"></span><span class=\"mord\"><span class=\"mord mathnormal\">a</span></span></span><span style=\"top:-2.41em;\"><span class=\"pstrut\" style=\"height:3em;\"></span><span class=\"mord\"><span class=\"mord mathnormal\">c</span></span></span></span><span class=\"vlist-s\">\u{200b}</span></span><span class=\"vlist-r\"><span class=\"vlist\" style=\"height:0.95em;\"><span></span></span></span></span></span><span class=\"arraycolsep\" style=\"width:0.5em;\"></span><span class=\"arraycolsep\" style=\"width:0.5em;\"></span><span class=\"col-align-c\"><span class=\"vlist-t vlist-t2\"><span class=\"vlist-r\"><span class=\"vlist\" style=\"height:1.45em;\"><span style=\"top:-3.61em;\"><span class=\"pstrut\" style=\"height:3em;\"></span><span class=\"mord\"><span class=\"mord mathnormal\">b</span></span></span><span style=\"top:-2.41em;\"><span class=\"pstrut\" style=\"height:3em;\"></span><span class=\"mord\"><span class=\"mord mathnormal\">d</span></span></span></span><span class=\"vlist-s\">\u{200b}</span></span><span class=\"vlist-r\"><span class=\"vlist\" style=\"height:0.95em;\"><span></span></span></span></span></span></span></span><span class=\"mclose\"><span class=\"delimsizing mult\"><span class=\"vlist-t vlist-t2\"><span class=\"vlist-r\"><span class=\"vlist\" style=\"height:1.45em;\"><span style=\"top:-3.45em;\"><span class=\"pstrut\" style=\"height:4.4em;\"></span><span style=\"width:0.333em;height:2.400em;\"><svg xmlns=\"http://www.w3.org/2000/svg\" width='0.333em' height='2.400em' viewBox='0 0 333 2400'><path d='M145 15 v585 v1200 v585 c2.667,10,9.667,15,21,15 c10,0,16.667,-5,20,-15 v-585 v-1200 v-585 c-2.667,-10,-9.667,-15,-21,-15 c-10,0,-16.667,5,-20,15z M188 15 H145 v585 v1200 v585 h43z'/></svg></span></span></span><span class=\"vlist-s\">\u{200b}</span></span><span class=\"vlist-r\"><span class=\"vlist\" style=\"height:0.95em;\"><span></span></span></span></span></span></span></span></span></span></span></span>",
        rendered_content
    );
}
