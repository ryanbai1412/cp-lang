// Codeforces 2207 E2: N-MEX (Counting Version).
// a_i is the largest of the n - i + 1 smallest values missing from b_1..b_i,
// so with a_0 = n it never rises and never falls below n - i. Given that, the
// count factors by step: where a steps down b_i has i choices, where it stays
// flat a_i - (n - i). (Product checked against brute force for n <= 5.)

type Z = Mod<1_000_000_007>

@!multitest
let n: 1..=200_000 = read()
domain Pos = 1..=n
let a: [Pos]0..=n = read()

fn choices(i: Pos) -> Int:
    let before = if let p = i.prev: a[p] else: n
    if not (n - i <= a[i] <= before): 0
    elif a[i] < before: i
    else: a[i] - (n - i)

print(Pos.product<Z>(choices))
