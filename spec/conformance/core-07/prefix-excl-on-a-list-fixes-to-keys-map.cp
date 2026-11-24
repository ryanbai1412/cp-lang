// expect: ok
// rule: 07.4.16
// tool: fix
// rewritten: "let xs = [2, 3, 4]\nlet pre = xs.scan(max)\nlet suf = xs.rev.scan(max)\nlet out = xs.keys.map(i => max(pre.cut(i), suf.cut(i + 1)))"
// With no named domain the rewrite maps the keys, reading both scans by count; a named op is called (decision 511).
let xs = [2, 3, 4]
let out = xs.prefix_excl(max)
