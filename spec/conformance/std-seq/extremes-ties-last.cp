// expect: ok, output "1 3\n2 0\n2 b 1 b\n9 4 -1"
// `ties: Ties.Last` picks the last of equal extremes in max, min, argmax and argmin (seq/U3).
let a = [3, 9, 2, 9]
print(a.argmax()!, a.argmax(ties: Ties.Last)!)
let b = [1, 5, 1]
print(b.argmin(ties: Ties.Last)!, b.argmin()!)
let p = [(2, 'a'), (1, 'b'), (2, 'b'), (1, 'a')]
print(p.max(by: q => q.0, ties: Ties.Last)!, p.min(by: q => q.0)!)
let e: List<Int> = []
print(a.keys.max(by: i => a[i], ties: Ties.Last).map(i => a[i])!, [4, 1].max(ties: Ties.Last)!, e.max(ties: Ties.Last) ?? -1)
