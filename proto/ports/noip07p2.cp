// NOIP 2007 String Expansion: a '-' between two digits or two lowercase
// letters, the right one larger, is replaced by the characters in between:
// as they are, in upper case or as '*' (p1), each repeated p2 times, in
// reverse when p3 is 2.
let (fill, repeat, order): (1..=3, 1..=8, 1..=2) = read()
let s: Str = read()

fn spans(l: Char, r: Char, lo: Char, hi: Char) -> Bool = lo <= l and l < r and r <= hi
fn shown(c: Char) -> Char:
    if fill == 3: return '*'
    if fill == 2 and c >= 'a': return 'A' + (c - 'a')
    c

var out = ""
for i in 0..s.len:
    if s[i] == '-' and 0 < i and i + 1 < s.len:
        let (l, r) = (s[i - 1], s[i + 1])
        if spans(l, r, '0', '9') or spans(l, r, 'a', 'z'):
            for k in 1..r - l:
                let c = if order == 1: l + k else: r - k
                for _ in 0..repeat: out.push(shown(c))
            continue
    out.push(s[i])
print(out)
