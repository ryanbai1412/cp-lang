//! Hover on names no declaration shows.

use super::*;

#[test]
fn hover_explains_the_types_and_values_the_compiler_defines() {
    let (mut server, _) =
        open("let x: Int = read()\nvar b: ?Int = None\nlet ys: List<Bool> = []\nprint(x, b, ys)\n");
    let shown = |server: &mut Server, line, character| {
        hover_at(server, line, character)
            .at(&["contents", "value"])
            .as_str()
            .unwrap_or("")
            .to_string()
    };
    assert!(shown(&mut server, 0, 8).contains("type Int"));
    assert!(shown(&mut server, 1, 15).contains("None: ?T"));
    assert!(shown(&mut server, 2, 9).contains("type List<T>"));
    assert!(shown(&mut server, 2, 15).contains("type Bool"));
}
