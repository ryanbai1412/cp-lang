// n: 512
// fixed: A
use std.strings.{is_palindrome, max_rotation}
let n: Int = read()
let abs = ["ab".repeat(n / 2) for _ in 0..2]
let pal = ["ab".repeat(n / 4) + "ba".repeat(n / 4) for _ in 0..2]
let rots = ["a".repeat(n - 1) + "b" for _ in 0..2]
let spaced = Str([if i % 5 == 4: ' ' else: 'x' for i in 0..n])
let digits = "0".repeat(n - 1) + "7"
let cs = [Char(Int('a') + i % 26) for i in 0..64]
var acc = 0
for i in 0..1 << 18:
    let c = cs[i % 64]
    acc += Int(c.lower) // timed: strings.lower
    acc += Int(c.upper) // timed: strings.upper
    acc += Char(Int('0') + i % 10).digit // timed: strings.digit
    if c.is_digit: acc += 1 // timed: strings.is_digit
    if c.is_lower: acc += 1 // timed: strings.is_lower
    if c.is_upper: acc += 1 // timed: strings.is_upper
    if c.is_alpha: acc += 1 // timed: strings.is_alpha
    if c.is_alnum: acc += 1 // timed: strings.is_alnum
    if c.is_space: acc += 1 // timed: strings.is_space
for r in 0..16384:
    let (ab, other) = (abs[r % 2], abs[1 - r % 2])
    if ab.starts_with(other): acc += 1 // timed: strings.starts_with [fixed: at]
    if ab.ends_with(other): acc += 1 // timed: strings.ends_with
    acc += common_prefix_len(ab, other) // timed: strings.common_prefix_len
    acc += common_suffix_len(ab, other) // timed: strings.common_suffix_len
    if is_palindrome(pal[r % 2]): acc += 1 // timed: strings.is_palindrome
    if let (a, b) = spaced.split_once('y'): acc += a.len // timed: strings.split_once
    acc += digits.parse_int() ?? 0 // timed: strings.parse_int
    acc += digits.parse<Int>() ?? 0 // timed: strings.parse
    let rot = rots[r % 2]
    acc += lyndon_factorization(rot).len // timed: strings.lyndon_factorization
    acc += min_rotation(rot) ?? 0 // timed: strings.min_rotation
    acc += max_rotation(rot) ?? 0 // timed: strings.max_rotation
print(acc)
