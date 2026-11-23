// Codeforces 2183 G — Interactive Shift Recovery
//
// The statement bounds all `n` initial positions `a[0..n]` to `Pos = 1..=4 * n`,
// and we keep every coordinate 1-based without shifting.
//
// Recover each shift `ans[i] in 0..=2` from three queries `"R"`, `"RL"`, `"L"`:
// - Every `v in vrl` is an element that survived shifting right then left, so
//   it returns to its original position `a[i] in Pos` and matches `r in vr` at
//   the same index, giving `ans[idx[Pos(v)]!] = r - v`.
// - Every `v in vl` is shifted left by `0..=2`, so `v <= 4 * n` always; the
//   only out-of-range values `"L"` can return are `v <= 0` (when `a[i] <= 2`
//   shifts left past `1`), which `if v >= 1:` drops before recording `vl_has`.
//
// Any remaining position `i` (resolved right-to-left) must have `ans[i] >= 1`:
// - If its right neighbour has `ans[i + 1] == 0` and `a[i + 1] == a[i] + 1`,
//   having the left neighbour also fixed at `ans[i - 1] == 0` with
//   `a[i - 1] == a[i] - 1` is ambiguous (`-1`); otherwise `a[i] - 1 in vl`
//   (tested directly against `vl_has[Pos(a[i] - 1)]`) distinguishes
//   `ans[i] == 1` from `ans[i] == 2`.
// - Otherwise `ans[i] == 2`.

@!interactive

fn ask(s: Str) -> List<Int>:
    print("?", s)
    let k: Int = read()
    let got: [0..k]Int = read()
    got.collect()

@!multitest
let n: 1..=200_000 = read()
domain Pos = 1..=4 * n
let a: [0..n]1..=4 * n = read()
let vr = ask("R")
let vrl = ask("RL")
let vl = ask("L")

var ans: [0..n]?(0..=2) = None
var idx: [Pos]?Int = None
for (i, x) in a.items: idx[Pos(x)] = i
for (v, r) in zip(vrl, vr): ans[idx[Pos(v)]!] = r - v

var vl_has: [Pos]Bool = false
for v in vl if v >= 1: vl_has[Pos(v)] = true

for i in (0..n).rev if ans[i] == None:
    if i + 1 < n and ans[i + 1] == 0 and a[i + 1] == a[i] + 1:
        if i > 0 and ans[i - 1] == 0 and a[i - 1] == a[i] - 1:
            print("!", -1)
            return
        ans[i] = if a[i] > 1 and vl_has[Pos(a[i] - 1)]: 1 else: 2
    else:
        ans[i] = 2

print("!", [x! for x in ans])
