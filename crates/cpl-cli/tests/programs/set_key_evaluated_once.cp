// `x[k] op= v` on a type with only `set` evaluates `k` once (R01.6.4):
// `k(3)` runs once per statement, in both engines.
var calls = 0
fn k(x: Int) -> Int:
    calls += 1
    x

var have: Counter<Int> = Counter(0)
have[k(3)] += 2
have[k(3)] max= 7
let a = [2, 2, 5]
for i in a.keys: have[a[i]] += 1
print(have[3], have[2], have[5], calls)
