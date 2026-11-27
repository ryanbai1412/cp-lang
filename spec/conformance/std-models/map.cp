// expect: ok, output "ok"
// input: "1 300\n"
// model: Map
// model: Set
// Map<Int, Int> and Set<Int> against a List<?Int> and a List<Bool> indexed by key: seeded random
// m[k] = v, m[k] += v, remove, pop_first, pop_last and clear, with get, contains, len, items,
// first, last and the four neighbour searches compared after each op. Reads `seed n`; prints
// the first op that differs.

/// A Lehmer generator: the case's own, so it needs no `std.rand`.
struct Lcg:
    var x: Int

impl Lcg:
    fn int(mut self, lo: Int, hi: Int) -> Int:
        x = x * 48271 % 2147483647
        lo + x % (hi - lo + 1)

fn key(i: Int) -> Int = 3 * i - 50

fn entries(a: List<?Int>) -> List<(Int, Int)>:
    var out: List<(Int, Int)> = []
    for i in 0..a.len:
        if let v = a[i]: out.push((key(i), v))
    out

/// The entry of `a` nearest slot `i` stepping by `d`, starting at `i` when `with`.
fn near(a: List<?Int>, i: Int, d: Int, with: Bool) -> ?(Int, Int):
    var j = if with: i else: i + d
    while j >= 0 and j < a.len:
        if let v = a[j]: return Some((key(j), v))
        j += d
    None

fn near_in(b: List<Bool>, i: Int, d: Int, with: Bool) -> ?Int:
    var j = if with: i else: i + d
    while j >= 0 and j < b.len:
        if b[j]: return Some(key(j))
        j += d
    None

let seed0 = read<Int>()
let n = read<Int>()
var rng = Lcg(seed0 % 2147483646 + 1)
var m = Map<Int, Int>()
var s = Set<Int>()
var a: List<?Int> = List.filled(40, None)
var b: List<Bool> = List.filled(40, false)
var bad = ""
var at = 0
for k in 0..n:
    let op = rng.int(0, 7)
    let i = rng.int(0, 39)
    let v = rng.int(-9, 9)
    if op <= 1:
        m[key(i)] = v
        a[i] = Some(v)
        let _ = s.add(key(i))
        b[i] = true
    elif op == 2:
        m[key(i)] += v
        a[i] = Some((a[i] ?? 0) + v)
    elif op == 3:
        if m.remove(key(i)) != a[i]: bad = "remove"
        a[i] = None
        if s.remove(key(i)) != b[i]: bad = "set remove"
        b[i] = false
    elif op == 4:
        let want = near(a, 0, 1, true)
        if m.pop_first() != want: bad = "pop_first"
        if let (x, _) = want: a[(x + 50) / 3] = None
    elif op == 5:
        let want = near(a, 39, -1, true)
        if m.pop_last() != want: bad = "pop_last"
        if let (x, _) = want: a[(x + 50) / 3] = None
        let top = near_in(b, 39, -1, true)
        if s.pop_last() != top: bad = "set pop_last"
        if let x = top: b[(x + 50) / 3] = false
    elif op == 6:
        if rng.int(0, 5) == 0:
            m.clear()
            a = List.filled(40, None)
            s.clear()
            b = List.filled(40, false)
    else:
        let low = near_in(b, 0, 1, true)
        if s.pop_first() != low: bad = "set pop_first"
        if let x = low: b[(x + 50) / 3] = false
    let e = entries(a)
    if m.len != e.len or [p for p in m.items] != e: bad = "items"
    if m.get(key(i)) != a[i] or m.contains(key(i)) != (a[i] != None): bad = "get"
    if m.first != near(a, 0, 1, true) or m.last != near(a, 39, -1, true): bad = "ends"
    if m.first_ge(key(i)) != near(a, i, 1, true) or m.first_gt(key(i)) != near(a, i, 1, false): bad = "above"
    if m.last_le(key(i)) != near(a, i, -1, true) or m.last_lt(key(i)) != near(a, i, -1, false): bad = "below"
    if s.contains(key(i)) != b[i] or s.len != b.count(x => x): bad = "set contains"
    if s.first_ge(key(i)) != near_in(b, i, 1, true) or s.last_lt(key(i)) != near_in(b, i, -1, false): bad = "set near"
    if bad != "":
        at = k
        break
if bad == "": print("ok") else: print("op", at, bad)
