// Source: SDOI 2009, School Canteen, https://dmoj.ca/problem/etsr23r2p4
// Problem: 1 <= C <= 5 tests. 1 <= n <= 1000 students queue; student i has taste
//   0 <= T_i <= 1000 and tolerance 0 <= B_i <= 7: at most the B_i people right behind i may be
//   served before i. Cooking a dish after one of taste a costs a xor b (the first is free).
//   Print the least total time.
// Solution: everyone before the first unserved student ci is served, and nobody beyond ci + 7
//   can be, so a state is (ci, last served student, bitmask of served students in the window
//   after ci). The next student is any unserved pos up to the least pos + B_pos over unserved
//   pos in the window; memoised recursion over a Map: O(n * 2^8 * 16 * 8) per test.
// original: corpus/submissions/5494510.cpp

@!multitest
let n: 1..=1000 = read()
struct Student:
    taste: 0..=1000
    patience: 0..=7
let queue: [n]Student = read()

var memo = Map<(Int, Int, Int), Int>()

fn dp(ind: Int, prv: Int, mask: Int) -> Int:
    var ci = ind
    var cm = mask
    while ci < n and (cm & 1) == 1:
        ci += 1
        cm >>= 1
    if ci >= n: return 0

    var last_pos = n - 1
    for pos in ci..min(n, ci + 15):
        if not cm.bit(pos - ci): last_pos min= pos + queue[pos].patience

    let key = (ci, prv, cm)
    if let v = memo.get(key): return v

    var best: ?Int = None
    for pos in ci..=last_pos:
        let bit = pos - ci
        if cm.bit(bit): continue
        let cost = if prv == -1: 0 else: queue[prv].taste ^ queue[pos].taste
        best min= cost + dp(ci, pos, cm | (1 << bit))
    memo[key] = best!
    best!

print(dp(0, -1, 0))
