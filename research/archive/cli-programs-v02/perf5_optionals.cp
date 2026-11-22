let n: Int = read()
domain V = 0..n
type Far = (depth: Int, child: V)
struct Best:
    far: ?Far
    other: ?Int

fn deeper(a: ?Int, b: ?Int) -> ?Int:
    if let x = a, let y = b: return max(x, y)
    a ?? b

fn pick(b: Best) -> Int:
    let top = b.far ?? return -1
    top.depth + top.child.idx

// `none`, 0 and the smallest Int but one.
let low = Int.MIN + 1
var plain: ?Int = none
print(plain, plain ?? 7, deeper(none, none), deeper(0, none), deeper(none, low), deeper(low, 0))
plain = 0
print(plain, plain ?? 7, plain == 0, plain == none)
plain = low
print(plain, plain!)
var slots: [V]?Int = none
slots[V.first] = 0
slots[V.last] = low
print(slots, [x ?? -1 for x in slots])

// `?V` and `[V]?V`.
var parent: [V]?V = none
for v in V if v > V.first: parent[v] = (v - 1)!
print(parent)
var at: ?V = V.last
var steps = 0
while let v = at:
    steps += 1
    at = parent[v]
print(steps, at, at == none)

// A tuple with a position in it, as an optional.
var far: ?Far = none
print(far, far == none, pick(Best(far, none)))
far = (3, V.first)
print(far, far == none, pick(Best(far, 4)))
if let f = far: print(f.depth, f.child)
var bests: List<Best> = List.filled(n, Best(none, none))
bests[1] = Best((5, V.last), low)
bests[2] = Best((0, V.first), 0)
for b in bests: print(b.far, b.other, pick(b))
print(bests[1].far == bests[2].far, bests[0].far == none, bests[2].far == (0, V.first))
var table: [V]?(V, Int) = none
table[V.last] = (V.first, low)
print(table, [t.1 for v in V if let t = table[v]])
var seen = Set<?Far>()
seen.add(none)
seen.add((1, V.first))
seen.add(none)
print(seen.len, none in seen, far in seen)

// An optional of an optional.
var deep: ?(?Int) = none
print(deep)
deep = plain
print(deep, deep == none)
let empty: ?Int = none
deep = empty
print(deep == none, if let inner = deep: (inner ?? -5) else: -9)
