// Niche-packed `?V`: a position, a Bool and a Char are held in one scalar
// with a spare value for `none`, through every way an optional is made,
// tested, opened, compared, hashed, printed and stored.
let n: 1..=20 = read()
domain V = 0..n
let s: Str = read()
let raw: Str = read()

struct Cell:
    at: ?V
    on: ?Bool
    ch: ?Char

fn step(v: ?V) -> ?V:
    if let u = v: return u + 1
    none

// ?V: positions, with `none` as -1.
var nxt: [V]?V = none
var ps: List<V> = []
for v in V:
    nxt[v] = v + 1
    ps.push(v)
print(nxt, nxt[V.first] ?? V.last, nxt[V.last] ?? V.first, nxt[V.last] == none, nxt[V.first] == nxt[V.first], nxt[V.first] == nxt[V.last])
print(step(V.first) ?? V.last, step(V.last) ?? V.first, step(none) == none, V.try(0) == none, V.try(n) == none, V.try_at(n - 1) ?? V.first)
print(V.first.prev == none, V.last.next == none, V.first.next ?? V.last, V.last.prev ?? V.first)
print(ps.first ?? V.last, ps.last ?? V.first)
var seen = 0
while true:
    let p = ps.pop() ?? break
    seen += p.idx
print(seen, ps.first == none, ps.last == none)
var order: List<?V> = [V.last, none, V.first, none]
print(order, order[0] == order[2], order[1] == order[3])
var by: HashMap<?V, Int> = []
var keys: Set<?V> = []
for o in order:
    by[o] += 1
    keys.insert(o)
print(by[none], by[V.first], keys.len, order[0] in keys)
var m: Map<Int, V> = []
m[1] = V.first
print(m.get(1) ?? V.last, m.get(2) == none)

// ?Bool: one byte, 0, 1 or `none`.
var flags: List<?Bool> = [none, false, true]
var bits: [V]?Bool = none
bits[V.first] = false
bits[V.last] = true
print(flags, bits, flags[0] == none, flags[1] ?? true, flags[2] ?? false, bits[V.first] == false, bits[V.first] == none)
var fset: Set<?Bool> = []
for f in flags: fset.insert(f)
print(fset.len, flags[2] in fset, flags.pop() ?? false, flags.first == none)

// ?Char: every byte is a Char, so `none` lives past them.
var cs: List<?Char> = [none]
for c in s: cs.push(c)
cs.push('~')
print(cs, cs[0] ?? '-', cs[1] ?? '-', cs[0] == none, cs[1] == 'a')
// Byte 255 is a Char like any other, and is not `none`.
var top: [V]?Char = none
top[V.first] = raw[1]
top[V.last] = raw[0]
print(Int(top[V.first] ?? 'x'), top[V.first] == none, Int(top[V.last] ?? 'x'), top[V.at(1)] == none, top[V.first] == top[V.last])

var cells: List<Cell> = [Cell(at: none, on: none, ch: none), Cell(at: V.last, on: false, ch: 'q')]
for c in cells:
    print(c.at ?? V.first, c.on ?? true, c.ch ?? '?', c.at == none, c.on == none, c.ch == none)
let pair: (?V, ?Bool, ?Char, ?Int) = (V.first, none, 'x', Int.MIN)
print(pair.0 ?? V.last, pair.1 ?? true, pair.2 ?? '-', pair.3 ?? 0, pair)

// A table that is passed, looped over or compared is used whole, and one
// fed from an unknown Int has no value to spare: both stay std::optional.
fn total(xs: List<?Int>) -> Int:
    var t = 0
    for x in xs: t += x ?? 1
    t
var whole: List<?Int> = [none, 2]
whole.push(-1)
var fed: [V]?Int = none
let k: Int = read()
fed[V.first] = k
fed[V.last] = k + 1
print(total(whole), whole, fed, fed[V.first] == Int.MIN, fed[V.last] ?? 0)
