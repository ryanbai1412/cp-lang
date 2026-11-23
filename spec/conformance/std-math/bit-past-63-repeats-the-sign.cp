// expect: ok, output "YES"
var ok = true
for x in [-1, -5, 0, 5, 1 << 62, -(1 << 62)]:
    for i in 0..130:
        let want = if i >= 63: x < 0 else: (x >> i) & 1 == 1
        ok = ok and x.bit(i) == want
print(yes(ok))
