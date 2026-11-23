// expect: ok, output "24"
// `while let` takes any pattern, a tuple included (round 22, H12).
var xs = [(2, "bb"), (1, "aaaa")]
var t = 0
while let (k, s) = xs.pop():
    t += k * s.len
    t *= 2
print(t)
