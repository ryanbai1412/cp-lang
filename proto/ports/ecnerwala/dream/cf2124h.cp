// Codeforces 2124 H — Longest Good Subsequence
//
// Prepend a dummy 0 at index 0 so the whole answer is a good subsequence
// starting at value a[0] = 0. Let `max_len[tidx(i, j)]` (stored in a lower
// triangle for i <= j, element type `0..=2 * n` so storage narrows to 16 bits)
// be the maximum value reachable by a good subsequence starting at i with
// value a[i] and ending at j with value a[j] == a[i].
//
// Scanning i from n - 1 down to 0 and j from i to n - 1 with running maximum
// `cur_best` (starting at a[i] - 1):
// - a[j] < a[i] cannot belong to a good subsequence starting at a[i].
// - a[j] == a[i] extends the current best by 1 and records `max_len[i, j]`.
// - a[i] < a[j] <= cur_best + 1 can branch into a sub-block starting at the
//   first occurrence `first_vis[a[j]]` and ending at j, updating `cur_best`
//   with `max_len[first_vis[a[j]], j]`.

fn tidx(i: Int, j: Int) -> Int = j * (j + 1) / 2 + i

@!multitest
let n0: 1..=15_000 = read()
let raw: [0..n0]1..=n0 = read()
let n = n0 + 1
let a = [0..n] i => if i == 0: 0 else: raw[i - 1]
var max_len: [0..n * (n + 1) / 2]0..=2 * n = 0

for i in (0..n).rev:
    let ai = a[i]
    var first_vis: [0..=n]?Int = none
    var cur_best = ai - 1
    for j in i..n:
        let aj = a[j]
        if aj < ai: continue
        if aj == ai:
            cur_best += 1
            max_len[tidx(i, j)] = cur_best
        elif aj <= cur_best + 1:
            first_vis[aj] ??= j
            if let fv = first_vis[aj]:
                cur_best max= max_len[tidx(fv, j)]
    if i == 0: print(cur_best)
