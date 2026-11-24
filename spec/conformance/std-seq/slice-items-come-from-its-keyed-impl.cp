// expect: ok, output "1 6
// 2 7
//
// 1 5
// 2 9"
// rule: std.seq.Keyed
// a slice's `items` is its `Keyed` impl's, which holds only when the source is `Indexed`
let a = [5, 6, 7]
print(a[1..].items.collect())
print([1, 2][2..].items.collect())
domain V = 0..4
let t: [V]Int = [4, 5, 9, 3]
for (v, x) in t[1..3].items: print(v.ord, x)
