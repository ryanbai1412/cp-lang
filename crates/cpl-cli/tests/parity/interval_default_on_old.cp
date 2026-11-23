domain P = 1..=8
var m = IntervalMap<P, ?Int>.new(None)
for (lo, hi, v) in [(2, 5, 3), (4, 7, 9)]:
    m.assign(P(lo)..=P(hi), v)
for (r, v) in m.each():
    print(r.len, v ?? 0)
