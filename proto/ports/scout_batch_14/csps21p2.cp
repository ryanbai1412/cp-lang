// CSP-S 2021 Bracket Sequences: count valid bracket-and-star strings matching
// a pattern with '?' wildcards, with at most k consecutive '*' in any run.
// original: corpus/submissions/4819734.cpp
const P = 1000000007
type Z = Mod<P>
let n: 1..=500 = read()
let k: 1..=500 = read() where k <= n
domain Pos = 0..n
let s: [Pos]Char = read()

fn ok(c: Char, target: Char) -> Bool = c == '?' or c == target

// wrap[l, r]: valid string enclosed by a matching '(' at l and ')' at r.
// seq[l, r]: any valid string on l..=r.
// star_seq[l, r]: 0..=k stars on l+1..=j followed by seq[j+1, r].
var wrap: [Pos, Pos]Z = 0
var seq: [Pos, Pos]Z = 0
var star_seq: [Pos, Pos]Z = 0

for l in Pos.rev:
    for r in l.r..:                                               // WISH P6
        if ok(s[l], '(') and ok(s[r], ')'):
            if r - l - 1 <= k and all(ok(s[i], '*') for i in l.r..r.l):   // WISH P6
                wrap[l, r] += 1
            for i in l.r..r.l if i - l - 1 <= k:                  // WISH P6, P9
                wrap[l, r] += seq[i, r - 1]                       // WISH P6
                if not ok(s[i], '*'): break
            for i in (l.r..r.l).rev if r - i - 1 <= k:            // WISH P6, P9
                if i < r - 1: wrap[l, r] += seq[l + 1, i]         // WISH P6
                if not ok(s[i], '*'): break
        for j in l..r if j - l <= k:                              // WISH P6, P9
            if j > l and not ok(s[j], '*'): break
            star_seq[l, r] += seq[j + 1, r]                       // WISH P6
        seq[l, r] = wrap[l, r] + sum(wrap[l, i] * star_seq[i, r] for i in l..r)

print(seq[Pos.first, Pos.last])
