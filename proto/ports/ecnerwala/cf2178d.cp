// Codeforces 2178 D: Xmas or Hysteria.
// Each surviving elf must kill at least one distinct elf, so `2 * m > n` is impossible.
// Sort elves by attack value in `ord`:
//   - If m >= 1: chain the `n - 2m` weakest elves into the next elf, then pair the
//     `m` strongest (`winners`) against the next `m` (`fodder`) so only `winners` survive.
//   - If m == 0: every elf must die, so find the largest index `s` whose suffix
//     `s..n - 1` sums to at least the strongest elf's attack; chain `0..s` into `s`,
//     then have `ord[s..n - 1]` attack `top` in descending order.

use std.seq.zip

@!multitest
let n: 2..=200_000 = read()
let m: 0..=200_000 = read()
domain Pos = 1..=n
let vals: [Pos]Int = read()
let ord = [p for p in Pos].sorted(by: p => vals[p])
let pref = prefix([0..n] i => vals[ord[i]])

fn plan() -> ?List<(Pos, Pos)>:
    if 2 * m > n: return none
    var pairs: List<(Pos, Pos)> = []
    if m >= 1:
        let cut = n - 2 * m
        for (a, b) in zip(ord[..cut], ord[1..=cut]): pairs.push((a, b))
        for (w, l) in zip(ord[n - m..].reversed(), ord[cut..n - m].reversed()): pairs.push((w, l))
    else:
        let top = ord[n - 1]
        let s = bsearch_last(0..n - 1, k => pref.sum(k..n - 1) >= vals[top]) ?? return none
        for (a, b) in zip(ord[..s], ord[1..=s]): pairs.push((a, b))
        for i in (s..n - 1).rev: pairs.push((ord[i], top))
    pairs

if let pairs = plan():
    print(pairs.len)
    for (a, b) in pairs: print(a, b)
else:
    print(-1)
