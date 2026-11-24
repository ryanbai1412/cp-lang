// D. Problem from Yuzhnouralsk
// https://codeforces.com/contest/2289/problem/D
// an operation at i swaps prefix sums S[i-1] and S[i]; all a >= 0 iff S is sorted and S[1] >= 0
@!multitest
let n: 2..=2e5 = read()
let a: [0..n]-1e9..=1e9 = read()
let s = prefix(a)
let sums = [s.cut(i) for i in 1..=n]
print(if sums.min()! < 0: -1 else: sums.inversions())
