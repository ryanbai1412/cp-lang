// expect: L0603 at 2
let a = [1, 2, 3]
let m = a.map(x => x * x)
print(m[0] + m[1] + m[2] + m[0] + m[1] + m[2])
