// Source: Codeforces 2147C (Rabbits), https://codeforces.com/contest/2147/problem/C
// Problem: a binary string of n <= 2e5 pots (sum n <= 2e5, multitest); each 0 gets a rabbit
//   facing left or right. A rabbit jumps into the next pot unless that pot holds a rabbit, is
//   past the border, or another rabbit faces it from the other side. Can no rabbit jump?
// Solution: "00" and "11" split s into independent maximal alternating runs. A 0 at a run's end
//   is free (border or rabbit beside it), and then every 0 of the run can be placed; a run
//   1010...1 has only isolated 0s, which must pair up facing a shared 1, so it needs an even
//   number of 0s. YES iff no run starts with 1 and has length 3 mod 4: one `runs` pass, O(n).
// Bounds: _n: 1..=200_000
@!multitest
let _n: 1..=200_000 = read()
let s: Str = read()
let ok = s.runs(link: (a, b) => a != b).all(r => r.first! != '1' or r.len % 4 != 3)
print(yes(ok))
