// expect: ok, output "24 9 7"
// A suffix table from `a.rev.scan(op)` holds its total at the first cut, so `.all` reads that one.
domain K = 0..3
let a = [K] d => d.ord + 2
print(a.rev.scan(*).all, a.rev.scan(+).all, a.rev.scan(+)[K(1).l])
