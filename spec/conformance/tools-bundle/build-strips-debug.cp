// expect: ok
// rule: bundle.2
// tool: build -o -
// has: "41"
// lacks: "x + 1);"
// `debug(...)` is not in the judge build (the source shows only as a comment).
let x = 41
debug(x + 1)
print(x)
