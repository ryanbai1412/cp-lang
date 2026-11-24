// The stdin twin of sum.cp's `@grader` fn: it declares none, so the
// package's grader.cpp is not linked in and it runs alone.
let n: 1..=100 = read()
let a: [n]Int = read()
print(a.sum())
