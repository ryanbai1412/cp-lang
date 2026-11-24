// expect: ok
// rule: lsp.2
// tool: lsp completion over: R|
// has: "label":"R"
// A fn's generic is in scope in its own signature, so typing it there offers it.
fn closes_to<R>(n: Int, over: R) -> Bool = n > 0
