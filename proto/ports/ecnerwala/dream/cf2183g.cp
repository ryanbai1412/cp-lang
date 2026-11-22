// Codeforces 2183 G — Interactive Shift Recovery
//
// Recover each shift `ans[i] in {0, 1, 2}` from three queries "R", "RL", "L".
// Comparing the matched positions in `vrl` (shifted right then left) against
// `vr` (shifted right) directly yields `ans[idx[v]] = r - v` for every element
// that survives "RL".
//
// Any remaining position `i` (resolved right-to-left) must have `ans[i] >= 1`:
// - If its right neighbour has `ans[i + 1] == 0` and `a[i + 1] == a[i] + 1`,
//   having the left neighbour also fixed at `ans[i - 1] == 0` with
//   `a[i - 1] == a[i] - 1` is ambiguous (`-1`); otherwise `a[i] - 1 in vl`
//   distinguishes `ans[i] == 1` from `ans[i] == 2`.
// - Otherwise `ans[i] == 2`.

@!interactive

fn ask(s: Str) -> List<Int>:
    print("?", s)
    let k: Int = read()
    let got: [0..k]Int = read()
    [x - 1 for x in got]

@!multitest
let n: 1..=200_000 = read()
let a_in: [0..n]Int = read()
let a = [0..n] i => a_in[i] - 1
let vr = ask("R")
let vrl = ask("RL")
let vl = ask("L")

let lim = max(4 * n, a[n - 1] + 2)
var ans: [0..n]?Int = none
var idx: [0..lim]?Int = none
for (i, x) in a.items: idx[x] = i
for (v, r) in zip(vrl, vr): ans[idx[v]!] = r - v

var vl_has: [0..lim]Bool = false
for v in vl if 0 <= v + 1 < lim: vl_has[v + 1] = true

for i in (0..n).rev if ans[i] == none:
    if i + 1 < n and ans[i + 1] == 0 and a[i + 1] == a[i] + 1:
        if i > 0 and ans[i - 1] == 0 and a[i - 1] == a[i] - 1:
            print("!", -1)
            return
        ans[i] = if vl_has[a[i]]: 1 else: 2
    else:
        ans[i] = 2

print("!", [x! for x in ans])
