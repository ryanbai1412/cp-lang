// expect: ok, output "2 1"
// rule: 01.6.4
// The key `a[i]` is evaluated once, even through `Counter`'s `set`.
let a = [2, 2, 5]
var have: Counter<Int> = Counter(0)
for i in a.keys: have[a[i]] += 1
print(have[2], have[5])
