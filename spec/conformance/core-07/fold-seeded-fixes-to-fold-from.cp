// expect: ok
// rule: 07.9.2
// tool: fix
// rewritten: "let a = [1, 2, 3]\nprint(a.fold_from(0, (s, x) => s * 10 + x), a.fold_from(1, (s, x) => s * x))"
// `cpl fix` rewrites the seeded `fold(f, init)` to `fold_from(init, f)`, init first (decision 550).
let a = [1, 2, 3]
print(a.fold((s, x) => s * 10 + x, 0), a.fold((s, x) => s * x, init: 1))
