// expect: ok, output "96"
// Every `for` clause takes its own `if`, not only the last (round 24, B1).
var t = 0
for a in 0..3 if a > 0, b in 0..3: t += a * 10 + b
print(t)
