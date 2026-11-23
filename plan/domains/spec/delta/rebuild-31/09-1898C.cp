// CF 1898C Colorful Grid
// Walk along row 1 then column m with alternating colours (n + m - 2 edges).
// Extra length must be even: each 4 is one lap around the cell at (1, 1)
// before leaving; a remaining 2 replaces the last step down by
// left, down, right around the cell at (n, m).
fn colour(i: Int) -> Char = if i % 2 == 0: 'R' else: 'B'

@!multitest
let (n, m, k): (3..=16, 3..=16, 1..=1e9) = read()
let base = n + m - 2
if k < base or (k - base) % 2 != 0:
    print("NO")
    return
domain R = 1..=n
domain C = 1..=m
domain HC = 1..m
domain VR = 1..n
var h: [R, HC]Char = 'R'
var v: [VR, C]Char = 'R'
for j in HC: h[R.first!, j] = colour(Int(j) - 1)
for i in VR: v[i, C.last!] = colour(m - 2 + Int(i))
v[VR(1), C(1)] = 'B'
h[R(2), HC(1)] = 'R'
v[VR(1), C(2)] = 'B'
if (k - base) % 4 == 2:
    h[R(n - 1), HC(m - 1)] = colour(base - 1)
    v[VR(n - 1), C(m - 1)] = colour(base - 2)
    h[R(n), HC(m - 1)] = colour(base - 1)
print("YES")
for i in R: print([h[i, j] for j in HC], sep: " ")
for i in VR: print([v[i, j] for j in C], sep: " ")
