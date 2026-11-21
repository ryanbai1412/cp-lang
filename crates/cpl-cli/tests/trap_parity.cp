// Every way a program can stop with a runtime error, one per `which`.
// `z` is read as 0 and `three` as 3, so nothing is known before the run.
let which: Int = read()
let z: Int = read()
let three: Int = read()
domain D = 0..three
domain E = 0..z
let xs = [1, 2, 3]
var fixed: [0..3]Int = 0
var grid: [0..3, 0..3]Int = 0
var table: [D]Int = 0
let s = "abc"
let nothing: ?Int = if z == 0: none else: 1
var st = Fenwick<D, Int>()
var out = 0
if which == 1: out = xs[three]
if which == 2: out = xs[z - 1]
if which == 3: out = fixed[three]
if which == 4: fixed[z - 1] = 1
if which == 5: out = grid[z, three]
if which == 6: grid[three, z] = 1
if which == 7: out = table[D(three)]
if which == 8: out = table[D(z - 1)]
if which == 9: out = s[three].ord
if which == 10: out = xs[2..three + 1].len
if which == 11: out = xs[three..1].len
if which == 12: out = nothing!
if which == 13: out = three / z
if which == 14: out = three % z
if which == 15: out = 1 << (three + 61)
if which == 16: out = 1 >> (z - 1)
if which == 17: out = 2 ** (z - 1)
if which == 18: out = Int.MIN / (z - 1)
if which == 19:
    var ys = [1, 2, 3]
    ys.swap(z, three)
if which == 20:
    var ys = [1, 2, 3]
    ys.truncate(z - 1)
if which == 21: out = s.repeat(z - 2).len
if which == 22: out = E.first.ord
if which == 23: out = E.last.ord
if which == 24:
    var ys: List<Int> = []
    out = ys.pop()!
if which == 25: assert z == 1, "stop"
if which == 26: out = st.sum(D(2)..D(z))
if which == 27: out = D.at(three).ord
if which == 28: out = Char(three + 253).ord
if which == 29: out = trunc(F64(three) * 1e18 * 10.0)
if which == 30: out = Int(I128(Int.MAX) + I128(three))
if which == 31: out = Int(U64(z - 1))
if which == 32: out = Int(U64(I128(z - 1)) >> 40)
if which == 33: out = table[(D.first + three)!]
if which == 34:
    let p: D = read()
    out = table[p]
if which == 35:
    let (a, b): (Int, Int) = read()
    out = a + b
if which == 36:
    let (a, on): (Int, Bool) = read()
    out = if on: a else: 0
print("fine", out)
