// expect: L0603
let a = [3, 1, 4, 1, 5]
let big = a.filter(x => x > 2) // <- here
for i in 0..3:
    print(big.sum() + i)
