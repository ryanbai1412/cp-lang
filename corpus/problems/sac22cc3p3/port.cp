// Source: SAC '22 Code Challenge 3 P3 (Bob Sort), https://dmoj.ca/problem/sac22cc3p3
// Problem: n <= 1e5 distinct A_i <= 1e9. For r = 1 .. (digits of max A), round r sorts the
//   array by the last r digits of each element (ties in any order); print the array
//   after each round.
// Solution: after round r the array is just sorted by A mod 10^r, so each round sorts the
//   input indices by (A mod 10^r, index) and prints; stop after the round where 10^r
//   passes max A. O(digits * n log n).
// original: corpus/submissions/4331598.cpp

let n: 1..=1000 = read()
let a: [0..n]Int = read()

var top = 0
for i in 0..n:
    top max= a[i]

// pass d orders by the last d digits; ties keep input order
var p = 10
var going = true
while going:
    var idx: List<Int> = []
    for i in 0..n:
        idx.push(i)
    idx.sort(by: i => (a[i] % p, i))
    var row: List<Int> = []
    for i in 0..n:
        row.push(a[idx[i]])
    print(row)
    going = top >= p
    p *= 10
