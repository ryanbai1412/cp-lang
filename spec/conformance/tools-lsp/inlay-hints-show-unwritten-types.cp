// expect: ok
// rule: lsp.8
// tool: lsp inlay
// has: "label":": List<Int>"
// has: "label":": Int"
// Inlay hints show the types the code does not write, on `xs` and on the loop variable.
let n = 3
let m: Int = 4
var xs = [n, m]
for x in xs:
    print(x)
