// expect: ok, output "6 5 3"
// rule: 04.2.4 04.2.5
// `V.cuts` is a domain, so it has `.range` like `V` does.
domain V = 0..5
print(V.cuts.range.len, V.range.len, Int(V.cuts.range.last_true(c => Int(c) <= 3)!))
