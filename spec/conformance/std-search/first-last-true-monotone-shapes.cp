// expect: ok, output "7 8 0"
// rule: std.search.split_best
// `last_true` takes true then false and returns the last true; `first_true` takes false then true (rebuild round 21 H3).
let r = 1..=10
print(r.last_true(k => k * k <= 50)!, r.first_true(k => k * k > 50)!, r.last_true(k => k > 99) ?? 0)
