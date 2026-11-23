// expect: ok, output "YES YES"
// rule: std.strings.RollingHash.combine
// `combine` takes any `b_len`, also one longer than the hashed sequence, so free-standing hashes need no dummy table.
let h = RollingHash([1, 2], bases: (31, 37))
let long = RollingHash([1, 2, 2, 2, 2, 2], bases: (31, 37))
let c = h.combine(h.get(0..1), long.get(1..6), 5)
print(yes(c == long.all), yes(h.combine(h.all, h.all, 2) == RollingHash([1, 2, 1, 2], bases: (31, 37)).all))
