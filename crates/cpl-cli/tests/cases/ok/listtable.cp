// cp: 0.1 b2d9838+dirty
let n: 1..=9 = read()
domain I = 1..=n
let a: [I]0..=100 = read()
let sq: [I]Int = [a[i] * a[i] for i in I]
let evens = [a[i] for i in I if a[i] % 2 == 0]
print(sq[I.at(n)], evens)
