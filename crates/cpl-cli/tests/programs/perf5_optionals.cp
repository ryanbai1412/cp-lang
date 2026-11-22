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

fn fmt_i(x: ?Int) -> Str = if let v = x: "{v}" else: "none"
fn fmt_v(x: ?V) -> Str = if let v = x: "{v.ord}" else: "none"
fn fmt_far(x: ?Far) -> Str = if let f = x: "{f.depth} {f.child.ord}" else: "none"
fn fmt_t(x: ?(V, Int)) -> Str = if let t = x: "{t.0.ord} {t.1}" else: "none"
fn fmt_deep(x: ?(?Int)) -> Str = if let inner = x: fmt_i(inner) else: "none"

// `none`, 0 and the smallest Int but one.
let low = Int.MIN + 1
var plain: ?Int = none
print(fmt_i(plain), plain ?? 7, fmt_i(deeper(none, none)), fmt_i(deeper(0, none)), fmt_i(deeper(none, low)), fmt_i(deeper(low, 0)))
plain = 0
print(fmt_i(plain), plain ?? 7, yes(plain == 0), yes(plain == none))
plain = low
print(fmt_i(plain), plain!)
var slots: [V]?Int = none
slots[V.first!] = 0
slots[V.last!] = low
print([fmt_i(x) for x in slots], [x ?? -1 for x in slots])

// `?V` and `[V]?V`.
var parent: [V]?V = none
for v in V if v > V.first!: parent[v] = v.prev!
print([fmt_v(x) for x in parent])
var at: ?V = V.last
var steps = 0
while let v = at:
    steps += 1
    at = parent[v]
print(steps, fmt_v(at), yes(at == none))

// A tuple with a position in it, as an optional.
var far: ?Far = none
print(fmt_far(far), yes(far == none), pick(Best(far, none)))
far = (3, V.first!)
print(fmt_far(far), yes(far == none), pick(Best(far, 4)))
if let f = far: print(f.depth, f.child.ord)
var bests: List<Best> = List.filled(n, Best(none, none))
bests[1] = Best((5, V.last!), low)
bests[2] = Best((0, V.first!), 0)
for b in bests: print(fmt_far(b.far), fmt_i(b.other), pick(b))
print(yes(bests[1].far == bests[2].far), yes(bests[0].far == none), yes(bests[2].far == (0, V.first!)))
var table: [V]?(V, Int) = none
table[V.last!] = (V.first!, low)
for i in 0..n - 1: print(fmt_t(table[V.at(i)!]))
print(fmt_t(table[V.last!]), [t.1 for v in V if let t = table[v]])
var seen = Set<?Far>()
seen.add(none)
seen.add((1, V.first!))
seen.add(none)
print(seen.len, yes(none in seen), yes(far in seen))

// An optional of an optional.
var deep: ?(?Int) = none
print(fmt_deep(deep))
deep = plain
print(fmt_deep(deep), yes(deep == none))
let empty: ?Int = none
deep = empty
print(yes(deep == none), if let inner = deep: (inner ?? -5) else: -9)
