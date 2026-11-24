// Source: NOIP 1999 Junior (Palindromic Numbers), https://dmoj.ca/problem/noip99p2
// Problem: a number M of at most 100 digits in base N (2 <= N <= 10 or N = 16). One step adds
//   the number to its own digit reversal in base N. Print `STEP=k` for the fewest steps that
//   give a palindrome, or `Impossible!` if it takes more than 30.
// Solution: simulate. Keep the digits as a list, test for a palindrome, else add the mirror
//   with carries; at most 31 tests of O(len) each.
// Note: mirror images by index arithmetic, as it has to be written today; builds, and
//   stress OK against 5044551.cpp.
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
    None
if let k = steps():
    print("STEP=", end: "")
    print(k)
else: print("Impossible!")
