// expect: ok, output "1 2\n3 6\n4 7"
// rule: 06.5.1
// input: "3\n2\n1 1\n3\n1 2 3\n1\n7\n"
// A shared `Map` keeps every case's keys while each case's `[n]` array starts fresh at its own size; interpreter and C++ agree.
var seen: Map<Int, Int> = Map()
@!multitest
let n: 1..=5 = read()
let a: [n]Int = read()
var cnt: [n]Int = 0
for x in a:
    seen[x] = (seen.get(x) ?? 0) + 1
for i in a.keys: cnt[i] += a[i]
print(seen.len, sum(cnt))
