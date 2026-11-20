// cp: 0.1 1f59ac0+dirty
// NOIP 2007 String Expansion: a '-' between two digits or two lowercase
// letters, the right one larger, is replaced by the characters in between:
// as they are, in upper case or as '*' (p1), each repeated p2 times, in
// reverse when p3 is 2.
let fill: 1..=3 = read()
let repeat: 1..=8 = read()
let order: 1..=2 = read()
let s: [domain S]Char = read(alphabet: "-abdx-159")

fn spans(l: Char, r: Char, lo: Char, hi: Char) -> Bool: lo <= l and l < r and r <= hi
fn shown(c: Char) -> Char:
    if fill == 3: return '*'
    if fill == 2 and c >= 'a': return 'A' + (c - 'a')
    c

var out: List<Char> = []
for i in S:
    if let before = i.prev and let after = i.next and s[i] == '-':
        let (l, r) = (s[before], s[after])
        if spans(l, r, '0', '9') or spans(l, r, 'a', 'z'):
            for k in 1..(r - l):
                let c = if order == 1: l + k else: r - k
                for _ in 0..repeat: out.push(shown(c))
            continue
    out.push(s[i])
print(out)
