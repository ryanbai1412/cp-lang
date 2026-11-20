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

    // p is a border of the prefix that ends before i. The longest border of
    // the prefix that ends after i: p or one of p's borders, grown by s[i].
    fn grow(p: Len, i: S) -> Len:
        var b = p
        while b != Len.first and s[b.right!] != s[i]: b = border[b]
        if s[b.right!] == s[i]: (b + 1)! else: b

    var half = Len.first                    // longest border of the prefix that fits in half of it
    var answer = Z(1)
    for i in S:
        let p = i.r
        depth[p] = 1
        if i != S.first:
            border[p] = grow(border[i.l], i)
            depth[p] = depth[border[p]] + 1
            half = grow(half, i)
        while half.idx * 2 > p.idx: half = border[half]
        answer *= Z(depth[half] + 1)
    print(answer)
