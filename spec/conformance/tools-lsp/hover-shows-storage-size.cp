// expect: ok
// rule: lsp.7
// tool: lsp hover x|s
// lacks: E0
// has: 1 B per element, 4 B max
// Hover on a variable gives its storage size (decision 149): four cells of `0..=255` take 1 byte each, 4 bytes in all.
let xs: [0..4]0..=255 = 7
print(xs[0])
