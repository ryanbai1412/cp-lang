// Codeforces 2089 D — Conditional Tree
//
// Parenthesize the binary string `s[0..2*half+1]` as a ternary `?:` expression
// evaluating to `'1'` (with `chain(lo, hi)` forming a right-associated chain).
//
// Three constructive cases cover all solvable strings:
// 1. `n >= 5` with `s[0..=1] == "10"` and `s[n - 1] == '1'`: group `(s[0]?s[1]:s[2])`
//    (evaluates to `'0'`) as the outer condition so it selects the final `s[n - 1] == '1'`.
// 2. `s[0] == '0'` and `s[n - 1] == '1'`: `0?chain(1, n - 1):1` directly selects `1`.
// 3. Otherwise we need an even-indexed `'1'` at `i` followed by an odd-indexed `'1'`
//    at `j > i` (so `s[i] == '1'` enters its true branch, which evaluates to `s[j] == '1'`,
//    while any leading `0`s skip over `chain(1, i)` straight to `s[i]`).

@!multitest
let half: 1..=200_000 = read()
let n = 2 * half + 1
let s: Str = read()

fn chain(lo: Int, hi: Int) -> Str:
    var e = "{s[lo]}"
    for i in lo + 1..hi by 2:
        e += "?{s[i]}:{s[i + 1]}"
    e

fn first_one(from: Int) -> ?Int:
    for k in from..n by 2 if s[k] == '1':
        return k
    none

fn build() -> ?Str:
    if n >= 5 and s[0] == '1' and s[1] == '0' and s[n - 1] == '1':
        return "({s[0]}?{s[1]}:{s[2]})?{chain(3, n - 1)}:{s[n - 1]}"
    if s[0] == '0' and s[n - 1] == '1':
        return "{s[0]}?{chain(1, n - 1)}:{s[n - 1]}"
    let i = first_one(0) ?? return none
    let j = first_one(i + 1) ?? return none
    var e = if i > 0: "{s[0]}?{chain(1, i)}:" else: ""
    e += "{s[i]}?"
    e += if j > i + 1: "({s[i + 1]}?{chain(i + 2, j)}:{s[j]})" else: "{s[j]}"
    e + ":{chain(j + 1, n)}"

if let e = build():
    print("Yes")
    print(e)
else:
    print("No")
