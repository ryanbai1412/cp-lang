// A word where a number is expected stops a local run; it is never read as 0.
let n: 1..=3 = read()
let xs: [0..n]Int = read()
print(sum(xs))
