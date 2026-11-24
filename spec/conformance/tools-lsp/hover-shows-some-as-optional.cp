// expect: ok
// rule: lsp.6
// tool: lsp hover s|q
// lacks: E0
// lacks: unknown
// has: ?Int
// Hover on a binding of `Some(x)` shows `?T` of `x`'s type, not an unknown.
let sq = Some(3)
print(sq!)
