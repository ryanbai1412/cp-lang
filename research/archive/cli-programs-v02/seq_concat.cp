let xs = [1, 2, 3]
let ys = [4, 5]
let arr: [0..3]Int = [10, 20, 30]
let brr: [0..2]Int = [40, 50]

print(xs + ys)
print(xs[1..] + ys)
print(xs + ys[..1])
print(xs[1..] + ys.rev)
print(arr + ys)
print(ys + arr)
print(arr + xs[1..])
print(xs[1..] + arr)
print(arr + brr)
print(arr + [])
print([] + arr)
print((arr + ys)[3])
print((xs[1..] + brr)[2])

let bits: [0..2]Bool = [true, false]
print(bits + [true])

let n = 4
domain V = 0..n
let pool: [V]V = [v for v in V]
let u = V.at(3)
print(pool[0..=1] + [u])
