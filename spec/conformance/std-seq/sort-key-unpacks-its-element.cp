// expect: ok, output "1 9\n3 5\n2 3\n3 5\n2 3\n1 9"
// rule: std.seq.sort
// A sort key's parameter is a pattern like any lambda's: it unpacks the element.
var ys = [(1, 9), (2, 3), (3, 5)]
ys.sort(by: ((a, b)) => -b)
var zs = [(1, 9), (2, 3), (3, 5)]
zs.sort(by: ((a, b)) => desc(a))
for (a, b) in ys: print(a, b)
for (a, b) in zs: print(a, b)
