// expect: ok
// rule: lsp.6
// tool: lsp hover x|s
// has: xs:
// has: [0..4]
// has: U8
// has: 0..=255
// Hover on a variable shows its declared type and the range its element is proven to hold.
let xs: [0..4]U8 = 7
print(xs[0])
