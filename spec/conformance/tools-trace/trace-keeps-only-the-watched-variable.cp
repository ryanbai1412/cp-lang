// expect: ok
// rule: trace.2
// tool: run --trace --watch t --format json
// has: "\"label\":\"t = s\""
// lacks: "\"label\":\"s += i\""
// `--watch t` keeps only the steps where `t` changes: the loop's per-iteration rows are left out.
var s = 0
var t = 0
for i in 0..5:
    s += i
t = s
print(t)
