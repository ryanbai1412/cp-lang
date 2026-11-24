// Source: ETSR 2023 R2 P2, https://dmoj.ca/problem/etsr23r2p2
// Problem: multitest; a digit string s of length n <= 24 and a target k <= 254. Insert `+` and
//   `*` between digits (usual precedence, numbers may have leading zeros) so the expression
//   equals k, using the fewest signs; print that count, or -1 if no placement works.
// Solution: values only matter up to CAP = 254: a value >= CAP stays >= CAP under `+` and under
//   `*` by a nonzero number, and a factor 0 makes the product 0 either way, so every value
//   saturates at CAP. prod[a][b][v] is the fewest `*` making s[a..=b] a product of value v (the
//   first factor is s[a..=m], the rest a product of s[m+1..=b]), O(n^3 * CAP). sum[c][v] is the
//   fewest signs making s[..c] a sum of products of value v, the last term being s[a..=c.l],
//   O(n^2 * CAP^2). The C++ sums over every interval instead, a factor n slower.
// original: corpus/submissions/5494433.cpp
const CAP = 254
domain Val = 0..=CAP
fn sat(x: Int) -> Val = Val(min(x, CAP))

@!multitest
let s: Str = read()
let k: Val = read()
domain Pos = 0..s.len

// number[a][b]: s[a..=b] read as one saturated number.
var number: [Pos][Pos]Val = Val(0)
for a in Pos:
    var x = Val(0)
    for b in a..=Pos.last!:
        x = sat(x.ord * 10 + (s[b.ord] - '0'))
        number[a][b] = x

var prod: [Pos][Pos][Val]?Int = None
for a in Pos.rev:
    for b in a..=Pos.last!:
        prod[a][b][number[a][b]] = 0
        for m in a..b:
            for v in Val:
                if let rest = prod[m.next!][b][v]:
                    let w = sat(number[a][m].ord * v.ord)
                    let cand = rest + 1
                    prod[a][b][w] = min(prod[a][b][w] ?? cand, cand)

var sum: [Pos.cuts][Val]?Int = None
for b in Pos:
    for v in Val:
        sum[b.r][v] = prod[Pos.first!][b][v]
    for a in Pos.first!.after:
        if a > b: break
        for head in Val:
            if let before = sum[a.l][head]:
                for v in Val:
                    if let term = prod[a][b][v]:
                        let w = sat(head.ord + v.ord)
                        let cand = before + 1 + term
                        sum[b.r][w] = min(sum[b.r][w] ?? cand, cand)
print(sum[Pos.cuts.last!][k], none: -1)
