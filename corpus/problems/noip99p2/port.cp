// Source: NOIP 1999 Junior (Palindromic Numbers), https://dmoj.ca/problem/noip99p2
// limits: 1000 ms, 64 MB
// Problem: a number M of at most 100 digits in base N (2 <= N <= 10 or N = 16). One step adds
//   the number to its own digit reversal in base N. Print `STEP=k` for the fewest steps that
//   give a palindrome, or `Impossible!` if it takes more than 30.
// Solution: simulate on the digit list. A number plus its reversal is symmetric in the digit
//   order, so the digits can be added from either end; at most 31 palindrome tests and sums of
//   O(len) each.
// Bounds: base: 2..=16
// original: corpus/submissions/5044551.cpp
use std.strings.{is_palindrome}

let base: 2..=16 = read()
let number: Str = read()

fn mirror_sum(d: List<Int>) -> List<Int>:
    var sum: List<Int> = []
    var carry = 0
    for (x, y) in zip(d, d.rev):
        carry += x + y
        sum.push(carry % base)
        carry /= base
    if carry > 0: sum.push(carry)
    sum

fn steps() -> ?0..=30:
    var d = [if c.is_digit: c - '0' else: c - 'A' + 10 for c in number]
    for k in 0..=30:
        if is_palindrome(d): return k
        d = mirror_sum(d)
    None

if let k = steps(): print("STEP={k}") else: print("Impossible!")
