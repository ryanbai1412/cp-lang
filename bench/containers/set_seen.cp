// A tree set asked whether a value is there just before it is added: the
// add answers that, so the set is walked once per value, not twice.
var s: Set<Int> = []
var seen = 0
var x = 987654321
for i in 0..2_000_000:
    x = (x * 1103515245 + 12345) % 2147483647
    let v = x % 1_000_000
    if s.contains(v): seen += 1
    s.add(v)
var h = 0
for v in s: h = (h * 31 + v) % 1_000_000_007
print(seen, s.len, h)
