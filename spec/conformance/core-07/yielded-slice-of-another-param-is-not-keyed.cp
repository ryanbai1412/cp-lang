// expect: ok, output "7\n8"
// rule: 07.4.5
// Only a fn whose generic `self` alone can give its slices keeps `self`'s keys: these are `other`'s.
fn of_other(self: List<Int>, other: List<Int>) -> yields Slice<List<Int>>:
    yield other[1..=2]
let xs = [1, 2]
let ys = [6, 7, 8]
for p in xs.of_other(ys):
    for k in p.keys: print(ys[k])
