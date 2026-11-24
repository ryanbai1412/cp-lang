// expect: ok, output "10 9 7 4 0"
// rule: 07.1.6 07.4.14
// A scan over `a.rev` keeps `a`'s cuts and stores at each the fold of everything right of it, the first one walked landing on the last cut.
domain V = 1..=4
let a = [V] v => Int(v)
let suf = a.rev.scan(+)
print(V.cuts.map(c => suf[c]))
