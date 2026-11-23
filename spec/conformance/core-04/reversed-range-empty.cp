// expect: ok, output "0"
// rule: 04.4.29
// A range whose start is past its end walks nothing, forwards or reversed.
let x = 3
var c = 0
for i in 5..=x: c += 1
for i in (5..=x).rev: c += 1
for i in 5..x: c += 1
print(c)
