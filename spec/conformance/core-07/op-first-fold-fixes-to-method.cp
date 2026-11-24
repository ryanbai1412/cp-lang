// expect: ok
// rule: 07.9.2
// tool: fix
// rewritten: "let a = [1, 2, 3]\nprint(a.fold(+), a.fold(max))"
// `cpl fix` rewrites the op-first `fold(op, xs)` to the method `xs.fold(op)`, op last (decision 550).
let a = [1, 2, 3]
print(fold(+, a), fold(max, a))
