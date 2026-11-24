// n: 512
// fixed: A
use std.strings.{is_palindrome, max_rotation, suffix_array, suffix_tree_edges, lyndon_runs, lcs_all}
let n: Int = read()
let ss = [Str([Char(Int('a') + (i * 7919 + i / 3) % 3) for i in 0..n]) for _ in 0..2]
let abs = ["ab".repeat(n / 2) for _ in 0..2]
let pal = ["ab".repeat(n / 4) + "ba".repeat(n / 4) for _ in 0..2]
let rots = ["a".repeat(n - 1) + "b" for _ in 0..2]
let half = abs[0][0..n / 2]
let miss = Str("ab".repeat(n / 4) + "c")
let spaced = Str([if i % 5 == 4: ' ' else: 'x' for i in 0..n])
let padded = " ".repeat(n / 2) + "x" + " ".repeat(n / 2)
let upper = "AB".repeat(n / 2)
let digits = "0".repeat(n - 1) + "7"
let words = [Str("abcd") for _ in 0..n / 4]
let wild = Str([if i % 2 == 0: 'a' else: '?' for i in 0..n / 8])
let cs = [Char(Int('a') + i % 26) for i in 0..64]
let ws = [Str([Char(Int('a') + (i * 7 + j) % 2) for i in 0..n / 4]) for j in 0..4]
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
for r in 0..2048:
    let (s, ab) = (ss[r % 2], abs[r % 2])
    acc += "a".repeat(n).len // timed: strings.repeat [fixed: self]
    acc += ab.search(miss) ?? 0 // timed: strings.search
    acc += ab.rsearch(miss) ?? 0 // timed: strings.rsearch
    acc += upper.to_lower().len // timed: strings.to_lower
    acc += ab.to_upper().len // timed: strings.to_upper
    acc += padded.trim().len // timed: strings.trim
    acc += padded.trim_start().len // timed: strings.trim_start
    acc += padded.trim_end().len // timed: strings.trim_end
    acc += spaced.split_whitespace().len // timed: strings.split_whitespace
    acc += ",".join(words).len // timed: strings.join
    acc += ab.replace("b", "cd").len // timed: strings.replace
    acc += z_function(s)[1] // timed: strings.z_function
    acc += prefix_function(s)[n - 1] // timed: strings.prefix_function
    for k in find_all(ab, half): acc += k // timed: strings.find_all
    acc += period(s) // timed: strings.period
    for b in borders(ab): acc += b // timed: strings.borders
    acc += ab.prefix_matches(half)[n - 1] // timed: strings.prefix_matches
    acc += overlap(ab, abs[1 - r % 2]) // timed: strings.overlap
    for k in ab.shift_and(half): acc += k // timed: strings.shift_and
    for k in ab.find_all_wildcard(wild): acc += k // timed: strings.find_all_wildcard
    acc += manacher(s).len // timed: strings.manacher
for r in 0..256:
    let s = ss[r % 2]
    acc += suffix_array(s).len // timed: strings.suffix_array
    acc += suffix_tree_edges(s).len // timed: strings.suffix_tree_edges
    acc += lyndon_runs(s).len // timed: strings.lyndon_runs
    acc += lcs_all<Char, Str>(ws) // timed: strings.lcs_all
print(acc)
