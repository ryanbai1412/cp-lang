// expect: ok, output "0 3 4"
// rule: 04.2.4
// `V.cuts` is a domain, so it has `.range` like any domain.
domain V = 0..3
let r = V.cuts.range
print(Int(r.first!), Int(r.last!), r.count())
