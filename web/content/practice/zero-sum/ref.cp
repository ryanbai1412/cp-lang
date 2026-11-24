let n: 1..=2e5 = read()
domain I = 0..n
let a: [I]-1e9..=1e9 = read()
let pre = a.scan(+)
var seen: Counter<Int, Int> = Counter()
var count = 0
for c in I.cuts:
    let s = pre.prefix(c)
    count += seen.get(s)
    seen[s] += 1
print(count)
