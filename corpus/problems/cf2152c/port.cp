// Source: Codeforces 2152C, Triple Removal, https://codeforces.com/contest/2152/problem/C
// limits: 2000 ms, 1024 MB
// Problem: multitest, binary array, n, q <= 250000 (sums bounded). An operation removes three
//   equal elements at i < j < k at cost min(j - i, k - j). For each query [l, r] print the
//   least cost to empty that subarray, or -1.
// Solution: possible iff the length and the number of ones are multiples of 3. Every triple
//   costs 1 if some adjacent pair is equal, and an alternating subarray pays 1 extra once.
//   Prefix sums of a_i and of a_i xor (i & 1) answer both checks in O(1) per query.

@!multitest
let (n, q): (1..=250_000, 1..=250_000) = read()
domain Pos = 1..=n
let a: [0..n]0..=1 = read()
let cnt_a = prefix(a)
let cnt_chk = prefix([a[i] ^ (i & 1) for i in 0..n])

for _ in 0..q:
    let (l_pos, r_pos): (Pos, Pos) = read()
    let l = l_pos.ord
    let r = r_pos.ord + 1
    let len = r - l
    if len % 3 != 0 or cnt_a.fold(l..r) % 3 != 0:
        print(-1)
    else:
        let raw = cnt_chk.fold(l..r)
        print(len / 3 + Int(min(raw, len - raw) == 0))
