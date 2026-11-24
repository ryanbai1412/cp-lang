fn find(self: List<Int>, x: Int) -> Int:
    for (i, y) in self.items:
        if y == x: return i
    -1

struct P:
    k: Int
    id: Int

let xs = [4, 7, 9]
let s = "hello"
print(xs.find(9) ?? -1, s.find("l") ?? -1, s.find("z") ?? -1)
var ps = [P(1, 0), P(2, 1), P(1, 2), P(2, 3), P(1, 4)]
ps.sort(by: p => desc(p.k))
print([p.id for p in ps])
var ts = [(1, 0), (2, 1), (1, 2), (2, 3)]
let us = ts.sorted(by: t => desc(t.0))
ts.sort(by: x => desc(x))
print(us, ts)
