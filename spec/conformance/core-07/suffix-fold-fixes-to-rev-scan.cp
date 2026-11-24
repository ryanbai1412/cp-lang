// expect: ok
// rule: 07.4.16
// tool: fix
// rewritten: "domain D = 0..3\nlet a: [D]Int = [2, 3, 4]\nlet s = a.rev.scan(+)\nprint(s.cut(1))"
// `cpl fix` rewrites the removed `suffix_fold(op, a)` to `a.rev.scan(op)`, on the same cuts (decision 511).
domain D = 0..3
let a: [D]Int = [2, 3, 4]
let s = suffix_fold(+, a)
print(s.cut(1))
