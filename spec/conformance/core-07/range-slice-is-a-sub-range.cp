// expect: ok, output "2 1 2 3 12 14 2 2 4 0 1 2 3 2 2 2 2"
// rule: 07.3.5
// A range is keyed by its values, so `r[a..b]` is the sub-range `a..b`, and std's `windows` cuts a domain's range with it.
domain V = 0..5
let s = V.all[V(1)..V(3)]
let t = (10..20)[12..=14]
print(s.len, s.first!.ord, s.last!.ord, t.len, t.first!, t.last!, (10..20)[..12].len, (10..20)[18..].len, V.windows(2).count(), [w.first!.ord for w in V.windows(2)], [w.len for w in V.windows(2)])
