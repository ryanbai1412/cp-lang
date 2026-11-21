// Codeforces 2178 D: Xmas or Hysteria.
// Each surviving elf must kill at least one distinct elf, so `2 * m > n` is impossible.
// Sort elves by attack value in `ord`:
//   - If m >= 1: chain the `n - 2m` weakest elves into the next elf, then pair each of
//     the `m` strongest against one of the next `m` weakest so exactly `m` survive.
//   - If m == 0: every elf must die, so find the largest index `s` whose suffix
//     `s..n - 1` sums to at least the strongest elf's attack; chain `0..s` into `s`,
//     then have `s..n - 1` attack the strongest elf in descending order.

@!multitest
let n: 2..=200_000 = read()
let m: 0..=200_000 = read()
domain Pos = 1..=n
let vals: [Pos]Int = read()
let ord = [p for p in Pos].sorted(by: p => vals[p])

if 2 * m > n:
    print(-1)
elif m >= 1:
    print(n - m)
    for i in 0..n - 2 * m: print(ord[i], ord[i + 1])
    for i in 0..m: print(ord[n - 1 - i], ord[n - 1 - m - i])
else:
    let pref = prefix([0..n] i => vals[ord[i]])
    let top = ord[n - 1]
    let split = bsearch_last(0..n - 1, k => pref.sum(k..n - 1) >= vals[top])
    if let s = split:
        print(n - 1)
        for i in 0..s: print(ord[i], ord[i + 1])
        for i in (s..n - 1).rev: print(ord[i], top)
    else:
        print(-1)
