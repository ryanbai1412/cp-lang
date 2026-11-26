// expect: ok, output "4\n8 9\n17\n6 9"
// rule: R07.4.6
let xs = [5, 6, 7, 8, 9]
let s = xs[1..5]
let t = s[3..5]
print(s.len)
print(t[3], t[4])
var total = 0
for x in t: total += x
print(total)
print(s.nth(0), t.nth(1))
