// A `for` over a sorted range, an I128 range value, a string's chars into a
// boxed variable, and a fed loop a closure runs eagerly all compile to the VM.
var st = Set<Int>()
for x in [1, 4, 6, 8, 9]: st.add(x)
for x in st.range(4..9): print(x)
for x in st.range(4..=9).rev: print(x)
for x in st.range(9..4): print(x)
var m = Map<Int, Str>()
m[2] = "b"
m[5] = "e"
m[7] = "g"
for (k, v) in m.range(3..): print(k, v)
for (k, v) in m.range(..=5).rev: print(k, v)
let r: Range<I128> = I128(3)..I128(6)
for v in r: print(v)
for v in r.rev: print(v)
fn gen(n: Int) -> yields Int:
    for i in 0..n: yield i * i
let f = (n: Int) => do:
    var t = 0
    for x in gen(n): t += x
    t
print(f(4))
