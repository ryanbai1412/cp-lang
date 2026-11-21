// `s.find_all(p)`, `st.fold(..)`, a guarded `while let`, and a literal that fills an array of arrays.
let n: Int = read()
let s = "abcabc"
print([i for i in s.find_all("bc")])

domain D = 0..4
var st = SegTree<D, Int>((a, b) => a + b)
st.set(D(1), 5)
st.set(D(3), 2)
print(st.fold(..), st.fold(D(2)..))

var xs = [1, 3, 2]
while let x = xs.pop(), x > 1:
    print(x)
if let z = xs.pop(), z > 0: print(z) else: print("empty")

var g: [0..n][0..4]Int = 0
g[1][3] = 5
print(g[1][3], g[0][3], g[0][0])
var f: [0..n][0..2]Bool = true
f[0][1] = false
print(f[0][1], f[1][1])
