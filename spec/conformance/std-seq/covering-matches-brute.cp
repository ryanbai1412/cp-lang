// expect: ok, output "YES"
use std.seq.{covering, pieces}
for _ in 0..200:
    let n = rand(0, 12)
    let cuts = [rand(0, n) for _ in 0..rand(0, 4)]
    let (domain P, piece) = (0..n).pieces(cuts: cuts)
    for a in 0..=n:
        for b in a..=n:
            var first: ?Int = None
            var last: ?Int = None
            for p in P:
                if Int(piece[p].l) == a: first = Some(p.to_idx)
                if Int(piece[p].r) == b: last = Some(p.to_idx + 1)
            if a == b and P.count > 0 and a == Int(piece[P.last!].r): first = Some(P.count)
            if a == b and P.count > 0 and a == Int(piece[P.first!].l): last = Some(0)
            if let i = first:
                if let j = last:
                    let got = piece.covering(a..b)
                    assert got.len == j - i
                    for p in got: assert i <= p.to_idx and p.to_idx < j
print(yes(true))
