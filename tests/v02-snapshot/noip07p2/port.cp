// Source: NOIP 2007 P2 (Expanding Strings), https://dmoj.ca/problem/noip07p2
// Problem: a string of <= 100 digits, lowercase letters and '-'. A '-' between two digits or two
//   letters, left < right, becomes the characters strictly between them: lowercase (p1 = 1),
//   uppercase (p1 = 2) or '*' (p1 = 3), each repeated p2 <= 8 times, reversed when p3 = 2.
//   Every other '-' stays. Print the expanded string.
// Solution: one pass: at an inner '-' whose neighbours qualify, build the run between them with
//   `shown` applying p1, reverse it for p3 = 2 and push each character p2 times; otherwise copy
//   the character. O(output length).
// original: corpus/submissions/5184882.cpp
let (p1, p2, p3): (1..=3, 1..=8, 1..=2) = read()
let s: Str = read()

fn same_kind(l: Char, r: Char) -> Bool = (l.is_digit and r.is_digit) or (l.is_lower and r.is_lower)

fn shown(c: Char) -> Char:
    if p1 == 3: return '*'
    if p1 == 2 and c.is_lower: return Char(c.ord - 'a'.ord + 'A'.ord)
    c

var out = Str()
for i in 0..s.len:
    let inner = s[i] == '-' and 0 < i and i + 1 < s.len
    if inner and s[i - 1] < s[i + 1] and same_kind(s[i - 1], s[i + 1]):
        var fill = [shown(Char(c)) for c in s[i - 1].ord + 1..s[i + 1].ord]
        if p3 == 2: fill.reverse()
        for c in fill:
            for _ in 0..p2: out.push(c)
    else:
        out.push(s[i])
print(out)
