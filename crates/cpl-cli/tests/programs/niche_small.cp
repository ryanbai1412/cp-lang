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

fn fmt_v(x: ?V) -> Str = if let v = x: "{v.ord}" else: "none"
fn fmt_b(x: ?Bool) -> Str = if let b = x: yes(b) else: "none"
fn fmt_c(x: ?Char) -> Str = if let c = x: "{c}" else: "none"
fn fmt_i(x: ?Int) -> Str = if let v = x: "{v}" else: "none"

fn step(v: ?V) -> ?V:
    if let u = v: return u.next
    none

// ?V: positions, with `none` as -1.
var nxt: [V]?V = none
var ps: List<V> = []
for v in V:
    nxt[v] = v.next
    ps.push(v)
print([fmt_v(x) for x in nxt], (nxt[V.first!] ?? V.last!).ord, (nxt[V.last!] ?? V.first!).ord, yes(nxt[V.last!] == none), yes(nxt[V.first!] == nxt[V.first!]), yes(nxt[V.first!] == nxt[V.last!]))
print((step(V.first!) ?? V.last!).ord, (step(V.last!) ?? V.first!).ord, yes(step(none) == none), yes(V.at(0) == none), yes(V.at(n) == none), (V.at(n - 1) ?? V.first!).ord)
print(yes(V.first!.prev == none), yes(V.last!.next == none), (V.first!.next ?? V.last!).ord, (V.last!.prev ?? V.first!).ord)
print((ps.first ?? V.last!).ord, (ps.last ?? V.first!).ord)
var seen = 0
while true:
    let p = ps.pop() ?? break
    seen += p.idx
print(seen, yes(ps.first == none), yes(ps.last == none))
var order: List<?V> = [V.last!, none, V.first!, none]
print([fmt_v(x) for x in order], yes(order[0] == order[2]), yes(order[1] == order[3]))
var by: HashMap<?V, Int> = []
var keys: Set<?V> = []
for o in order:
    by[o] += 1
    keys.insert(o)
print(by[none], by[V.first!], keys.len, yes(order[0] in keys))
var m: Map<Int, V> = []
m[1] = V.first!
print((m.get(1) ?? V.last!).ord, yes(m.get(2) == none))

// ?Bool: one byte, 0, 1 or `none`.
var flags: List<?Bool> = [none, false, true]
var bits: [V]?Bool = none
bits[V.first!] = false
bits[V.last!] = true
print([fmt_b(x) for x in flags], [fmt_b(x) for x in bits], yes(flags[0] == none), yes(flags[1] ?? true), yes(flags[2] ?? false), yes(bits[V.first!] == false), yes(bits[V.first!] == none))
var fset: Set<?Bool> = []
for f in flags: fset.insert(f)
print(fset.len, yes(flags[2] in fset), yes(flags.pop() ?? false), yes(flags.first == none))

// ?Char: every byte is a Char, so `none` lives past them.
var cs: List<?Char> = [none]
for c in s: cs.push(c)
cs.push('~')
print([fmt_c(x) for x in cs], cs[0] ?? '-', cs[1] ?? '-', yes(cs[0] == none), yes(cs[1] == 'a'))
// Byte 255 is a Char like any other, and is not `none`.
var top: [V]?Char = none
top[V.first!] = raw[1]
top[V.last!] = raw[0]
print(Int(top[V.first!] ?? 'x'), yes(top[V.first!] == none), Int(top[V.last!] ?? 'x'), yes(top[V.at(1)!] == none), yes(top[V.first!] == top[V.last!]))

var cells: List<Cell> = [Cell(at: none, on: none, ch: none), Cell(at: V.last!, on: false, ch: 'q')]
for c in cells:
    print((c.at ?? V.first!).ord, yes(c.on ?? true), c.ch ?? '?', yes(c.at == none), yes(c.on == none), yes(c.ch == none))
let pair: (?V, ?Bool, ?Char, ?Int) = (V.first!, none, 'x', Int.MIN)
print((pair.0 ?? V.last!).ord, yes(pair.1 ?? true), pair.2 ?? '-', pair.3 ?? 0, fmt_v(pair.0), fmt_b(pair.1), fmt_c(pair.2), fmt_i(pair.3))

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
fed[V.first!] = k
fed[V.last!] = k + 1
print(total(whole), [fmt_i(x) for x in whole], [fmt_i(x) for x in fed], yes(fed[V.first!] == Int.MIN), fed[V.last!] ?? 0)
