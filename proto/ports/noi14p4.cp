// cp: 0.1 f5fd87a
// NOI 2014 Zoo: for every prefix, count its borders (a proper prefix that is
// also a suffix) that are at most half as long; print the product of
// (count + 1) over all prefixes.
const P = 1000000007
type Z = Mod<P>

@multitest
fn solve():
    let s: [domain S]Char = read()
    type Len = S.bounds                     // a prefix, named by where it ends

    // border[p]: the longest border of prefix p; depth[p]: how many borders
    // p has, counting p itself (the border chain p, border[p], ...)
    var border: [Len]Len = Len.first
    var depth: [Len]Int = 0
    var half = Len.first                    // longest border of the prefix that fits in half of it
    var answer = Z(1)
    for i in S:
        let p = i.after
        depth[p] = 1
        if p.idx >= 2:
            var b = border[i.before]
            while b.idx > 0 and s[S(b.idx)] != s[i]: b = border[b]
            if s[S(b.idx)] == s[i]: b = Len(b.idx + 1)
            border[p] = b
            depth[p] = depth[b] + 1
        while half.idx > 0 and s[S(half.idx)] != s[i]: half = border[half]
        if p.idx >= 2 and s[S(half.idx)] == s[i]: half = Len(half.idx + 1)
        while half.idx * 2 > p.idx: half = border[half]
        answer *= Z(depth[half] + 1)
    print(answer)
