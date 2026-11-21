@!multitest
let half: Int = read()
let n = 2 * half + 1
let s: Str = read()

fn chain(lo: Int, hi: Int) -> Str:
    var e = "{s[lo]}"
    for i in (lo + 1..hi).step(2):
        e += "?{s[i]}:{s[i + 1]}"
    e

fn first_one(from: Int) -> ?Int:
    for k in (from..n).step(2) if s[k] == '1':
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
