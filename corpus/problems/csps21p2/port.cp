// CSP-S 2021 Bracket Sequences: count valid bracket-and-star strings matching
// a pattern with '?' wildcards, with at most k consecutive '*' in any run.
// original: corpus/submissions/4819734.cpp
const P = 1000000007
type Z = Mod<P>
let n: 1..=500 = read()
let k: 1..=500 = read()
assert k <= n
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
    for r in l..:
        if r == l: continue
        if ok(s[l], '(') and ok(s[r], ')'):
            let inner = r.idx - l.idx - 1
            if inner <= k:
                var all_star = true
                for i in l..r:
                    if i > l and not ok(s[i], '*'): all_star = false
                if all_star: wrap[l, r] += 1
            if let r1 = Pos.at(r - 1):
                for i in l..r:
                    if i == l: continue
                    if i.idx - l.idx - 1 > k: break
                    wrap[l, r] += seq[i, r1]
                    if not ok(s[i], '*'): break
            if let l1 = Pos.at(l + 1):
                for i in (l..r).rev:
                    if i == l: continue
                    if r.idx - i.idx - 1 > k: break
                    if i.idx < r.idx - 1: wrap[l, r] += seq[l1, i]
                    if not ok(s[i], '*'): break
        for j in l..r:
            if j.idx - l.idx > k: break
            if j > l and not ok(s[j], '*'): break
            if let j1 = Pos.at(j + 1): star_seq[l, r] += seq[j1, r]
        var total = wrap[l, r]
        for i in l..r: total += wrap[l, i] * star_seq[i, r]
        seq[l, r] = total

print(seq[Pos.first!, Pos.last!])
