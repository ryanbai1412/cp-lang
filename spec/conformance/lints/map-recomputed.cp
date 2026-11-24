// expect: L0603
let a = [1, 2, 3]
let m = a.map(x => x * x) // <- here
print(m[0] + m[1] + m[2] + m[0] + m[1] + m[2])
