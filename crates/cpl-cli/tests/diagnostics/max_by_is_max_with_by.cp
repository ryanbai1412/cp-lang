let p: 1..=10 = read()
let a: [0..p]Int = read()
let j = (0..p).max_by(i => a[i])
print(j)
