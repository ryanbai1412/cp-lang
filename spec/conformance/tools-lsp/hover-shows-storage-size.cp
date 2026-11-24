// expect: ok
// rule: lsp.7
// tool: lsp hover x|s
// has: 4 B
// Hover on a variable gives its storage size (decision 149): four `U8` cells take 4 bytes.
let xs: [0..4]U8 = 7
print(xs[0])
