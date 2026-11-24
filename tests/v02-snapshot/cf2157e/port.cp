// Source: Codeforces 2157E (Adjusting Drones), https://codeforces.com/contest/2157/problem/E
// Problem: n drones with energies a_i <= 2n and a limit k. While some energy is held by more
//   than k drones, every drone whose energy already appears at a smaller index gains 1. Print
//   the number of such operations. Multitest, sum n <= 2e5.
// Solution: each operation leaves one drone of a pile behind and moves the rest up, so a pile
//   drops a drone on each free level it passes. Going from level 2n down, keep a stack of the
//   levels not yet taken (fresh ones past 2n when it is empty); the copies of v take the
//   nearest ones, and the pile is down to k on the level of its (cnt - k + 1)-th copy, after
//   slot - v operations. Print the max over v: O(n).

@!multitest
let (n, k): (1..=200_000, 1..=200_000) = read()
domain Val = 1..=2 * n
let a: [0..n]Val = read()
var freq: [Val]Int = 0
for x in a: freq[x] += 1

var stk: List<Int> = []
var next_free = 2 * n
var ans = 0
for v in Val.rev:
    stk.push(v.ord)
    var rem = freq[v]
    while rem > 0:
        let slot = if let s = stk.pop():
            s
        else:
            let s = next_free
            next_free += 1
            s
        if rem == k: ans max= slot - v.ord
        rem -= 1
print(ans)
