// NOIP 2007 String Expansion: in a word of digits, lowercase letters and
// dashes, a dash between two digits or two letters, with the left one
// smaller, is replaced by the characters strictly between them. p1 picks
// their case (1 lower, 2 upper, 3 stars), p2 how often each is repeated, and
// p3 = 2 reverses the filled run. Every other dash stays.
//
// The original builds the run, rewrites it three times and appends it. Here
// the three parameters are three small steps on one generator.
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
