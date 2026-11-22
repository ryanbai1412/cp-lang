// cp: 0.1 bea787c+dirty
let s = "ab"
var t = s.copy()
t.push('c')
let q: List<(Int, Int)> = [(1, 2)]
var q2 = q
let (r, c) = q2[0]
q2.push((3, 4))
print(s, t, r, c)
