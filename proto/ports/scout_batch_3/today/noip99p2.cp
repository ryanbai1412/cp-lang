// NOIP 1999 Palindromic numbers, as it has to be written today: mirror
// images by index arithmetic. Builds; stress OK against 5044551.cpp.
let base: 2..=16 = read()
let word: [domain W]Char = read()
fn value(c: Char) -> Int:
    if c <= '9': c - '0' else: c - 'A' + 10
var digits: List<Int> = [value(c) for c in word]
fn mirror_sum(a: List<Int>) -> List<Int>:
    var out: List<Int> = []
    var carry = 0
    for i in 0..a.len:
        carry += a[i] + a[a.len - 1 - i]
        out.push(carry % base)
        carry /= base
    if carry > 0: out.push(carry)
    out
fn steps() -> ?Int:
    var d = digits
    for k in 0..=30:
        if all(d[i] == d[d.len - 1 - i] for i in 0..d.len): return k
        d = mirror_sum(d)
    none
if let k = steps():
    print("STEP=", end: "")
    print(k)
else: print("Impossible!")
