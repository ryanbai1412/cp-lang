let n: 1..=100 = read()
let a: [0..n]Int = read()
var c = 0
for x in a:  // expect: loop_reduction
    if x % 2 == 0:
        c += 1
print(c)
