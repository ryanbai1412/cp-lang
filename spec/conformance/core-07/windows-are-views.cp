// expect: ok, output "3 5"
// rule: 07.7.9
// `windows(k)` yields views of the source, each `k` long (rebuild 7, H-WINDOWS-COST).
print([1, 2, 3].windows(2).map(w => w.sum()))
