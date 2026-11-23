let n = 3
let a = read<[n]Int>()
let p = a.scan(+)
print(p[n])
