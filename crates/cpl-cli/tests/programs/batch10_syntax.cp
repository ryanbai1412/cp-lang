// A member that reads with no arguments reads the same with empty parentheses, on a
// `Set` and a `Map` too; `sorted` takes a range written out.
domain E = 0..4
let w: [E]Int = [E] e => (e.ord * 3) % 4
var s: Set<Int> = Set()
var m: Map<Int, Str> = Map()
for x in [5, 2, 9]:
    s.insert(x)
    m[x] = "v{x}"
print(s.first(), s.last(), s.first, s.len())
print(m.first(), m.last())
print(sorted(E.all, by: e => w[e]))
print(sorted(0..5, by: i => (i * 2) % 5))
print(sorted(1..=3, by: i => -i))
