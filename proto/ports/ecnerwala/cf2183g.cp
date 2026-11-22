// Codeforces 2183 G — Interactive Shift Recovery
//
// Positions `a[0..n]` lie in `Pos = 1..=4 * n + 1` (the statement bounds all
// positions to `<= 4n`, with `+ 1` room for `n = 2` generator instances), and
// we keep every coordinate 1-based without shifting.
//
// Recover each shift `ans[i] in 0..=2` from three queries `"R"`, `"RL"`, `"L"`.
// Comparing the matched positions in `vrl` (shifted right then left) against
// `vr` (shifted right) directly yields `ans[idx[Pos(v)]!] = r - v` for every
// element that survives `"RL"`.
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
    [x for x in got]

@!multitest
let n: 1..=200_000 = read()
let bound = 4 * n + 2
domain Pos = 1..=bound
let a: [0..n]1..=bound = read()
let vr = ask("R")
let vrl = ask("RL")
let vl = ask("L")

var ans: [0..n]?(0..=2) = none
var idx: [Pos]?Int = none
for (i, x) in a.items: idx[Pos(x)] = i
for (v, r) in zip(vrl, vr): ans[idx[Pos(v)]!] = r - v

var vl_has: [Pos]Bool = false
for v in vl if 1 <= v <= bound: vl_has[Pos(v)] = true

for i in (0..n).rev if ans[i] == none:
    if i + 1 < n and ans[i + 1] == 0 and a[i + 1] == a[i] + 1:
        if i > 0 and ans[i - 1] == 0 and a[i - 1] == a[i] - 1:
            print("!", -1)
            return
        ans[i] = if a[i] > 1 and vl_has[Pos(a[i] - 1)]: 1 else: 2
    else:
        ans[i] = 2

print("!", [x! for x in ans])
