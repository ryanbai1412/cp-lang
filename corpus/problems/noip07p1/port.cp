// NOIP 2007 Counting Numbers: list each distinct number with how often it
// occurs, in increasing order.
let n: 1..=200000 = read()
let xs: [0..n]0..=1500000000 = read()
for (x, times) in runs(xs.sorted()): print(x, times)
