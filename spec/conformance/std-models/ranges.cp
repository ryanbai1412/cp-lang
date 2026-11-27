// expect: ok, output "ok"
// input: "1 300\n"
// model: range_of
// model: slice
// model: slice_ends
// model: sub_range
// model: sub_range_ends
// model: some
// `lo..hi by st` against a list of its values built by a loop, and `xs[a..b]`, `xs[a..]`,
// `xs[..b]`, `xs[..=b]` of a list and `q[a..b]`, `q[a..]`, `q[..=b]` of a range against copies
// made index by index, for seeded random ends. Reads `seed n`; prints the first that differs.

/// A Lehmer generator: the case's own, so it needs no `std.rand`.
struct Lcg:
    var x: Int

impl Lcg:
    fn int(mut self, lo: Int, hi: Int) -> Int:
        x = x * 48271 % 2147483647
        lo + x % (hi - lo + 1)

/// `f(i)` for each `i` in `lo..hi`, by a loop.
fn run(lo: Int, hi: Int, f: (Int) -> Int) -> List<Int>:
    var out: List<Int> = []
    var i = lo
    while i < hi:
        out.push(f(i))
        i += 1
    out

let seed0 = read<Int>()
let n = read<Int>()
var rng = Lcg(seed0 % 2147483646 + 1)
let xs = run(0, 20, i => 3 * i - 7)
let q = 0..20
var bad = ""
var at = 0
for k in 0..n:
    let lo = rng.int(-5, 15)
    let hi = rng.int(-5, 15)
    let st = rng.int(1, 4)
    var want: List<Int> = []
    var v = lo
    while v < hi:
        want.push(v)
        v += st
    let r = lo..hi by st
    if List(r) != want or r.len != want.len: bad = "range"
    let a = rng.int(0, 20)
    let b = rng.int(a, 20)
    if List(xs[a..b]) != run(a, b, i => xs[i]): bad = "slice"
    if List(xs[a..]) != run(a, 20, i => xs[i]): bad = "slice from"
    if List(xs[..b]) != run(0, b, i => xs[i]): bad = "slice to"
    if b < 20 and List(xs[..=b]) != run(0, b + 1, i => xs[i]): bad = "slice through"
    if List(q[a..b]) != run(a, b, i => i): bad = "sub range"
    if List(q[a..]) != run(a, 20, i => i): bad = "sub range from"
    if b < 20 and List(q[..=b]) != run(0, b + 1, i => i): bad = "sub range through"
    let o = Some(a)
    if o != Some(a) or o == None: bad = "some"
    if bad != "":
        at = k
        break
if bad == "": print("ok") else: print("op", at, bad)
