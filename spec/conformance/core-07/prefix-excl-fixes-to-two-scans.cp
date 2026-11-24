// expect: ok
// rule: 07.4.16
// tool: fix
// rewritten: "domain D = 0..3\nlet a: [D]Int = [2, 3, 4]\nlet pre = a.scan(*)\nlet suf = a.rev.scan(*)\nlet out = [D] i => pre[i.l] * suf[i.r]\nprint(out)"
// `cpl fix` rewrites the removed `a.prefix_excl(op)`: two scans hoisted above the statement, so the fold of everything but `i` stays O(n) (decision 511).
domain D = 0..3
let a: [D]Int = [2, 3, 4]
let out = a.prefix_excl(*)
print(out)
