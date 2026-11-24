// expect: ok
// rule: lsp.6
// tool: lsp hover x|s
// lacks: E0
// has: xs:
// has: [0..4]
// has: 0..=255
// Hover on a variable shows its declared type and the range its element is proven to hold.
let xs: [0..4]0..=255 = 7
print(xs[0])
