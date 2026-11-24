// expect: ok
// rule: 07.9.2
// tool: fix
// rewritten: "let f = (a: Int, b: Int) => a * 10 + b\nlet a = [3, 1, 2]\nprint(a.reduce(f)!, a.reduce((x, y) => x - y)!)"
// `fold(op)` takes only an op with an identity, so `cpl fix` rewrites a fold by a plain fn to `reduce` (decision 550).
let f = (a: Int, b: Int) => a * 10 + b
let a = [3, 1, 2]
print(a.fold(f)!, a.fold((x, y) => x - y)!)
