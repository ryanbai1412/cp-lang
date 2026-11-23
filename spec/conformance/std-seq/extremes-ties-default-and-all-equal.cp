// expect: ok, output "0 0 a a\n3 3 d d"
// rule: 07.9.3
// Extremes take the first of equal elements by default; `ties: Ties.Last` on an all-equal list takes the last.
let a = [7, 7, 7, 7]
let p = [(1, 'a'), (1, 'b'), (1, 'c'), (1, 'd')]
print(a.argmax()!, a.argmin()!, p.max(by: q => q.0)!.1, p.min(by: q => q.0)!.1)
print(a.argmax(ties: Ties.Last)!, a.argmin(ties: Ties.Last)!, p.max(by: q => q.0, ties: Ties.Last)!.1, p.min(by: q => q.0, ties: Ties.Last)!.1)
