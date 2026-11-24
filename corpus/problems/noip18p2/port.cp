// Source: NOIP '18 P2 (Currency System), https://dmoj.ca/problem/noip18p2
// Problem: T test cases; n <= 100 coin values a_i <= 25000, each usable any number of
//   times. Print the fewest denominations of a system that pays exactly the same amounts.
// Solution: a coin is needed iff the smaller coins cannot pay it. Sort, and keep an
//   unbounded-knapsack table of payable amounts 0..25000: a coin not yet payable is
//   counted and added to the table. O(n * 25000) per test.
// original: corpus/submissions/4002899.cpp
const TOP = 25000
domain Amount = 0..=TOP

@!multitest
let n: 1..=100 = read()
let coins: [n]1..=TOP = read()
var payable: [Amount]Bool = false
payable[Amount(0)] = true
var needed = 0
for c in coins.sorted():
    if not payable[Amount(c)]:
        needed += 1
        for a in Amount:
            if let b = Amount.at(a - c) and payable[b]: payable[a] = true
print(needed)
