// expect: ok, output "45 45 10"
// rule: 05.10.12
// A `@get` that gives a stored field, or an element of one, lends it: each read is in place.
struct Rows:
    xs: List<Int>
    grid: [0..2]List<Int>
    @get pub fn items(self) -> List<Int> = xs
    @get pub fn first(self) -> List<Int> = grid[0]
let r = Rows([i for i in 0..10], [[i * 2 for i in 0..5], [1]])
var s = 0
for k in 0..10: s += r.items[k]
var t = 0
for k in 0..5: t += r.first[k] + r.first.len
print(s, t, r.items.len)
