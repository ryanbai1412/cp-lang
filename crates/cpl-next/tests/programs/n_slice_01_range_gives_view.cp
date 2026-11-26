// expect: ok, output "3 7 6\n2 8 2\n2\n3\n0 5 5"
// rule: R07.4.6
let xs = [5, 6, 7, 8, 9]
let s = xs[1..4]
print(s.len, s[2], s.nth(0))
let t = s[2..=3]
print(t.len, t.nth(1), t.keys.len)
for k in t.keys: print(k)
for x in xs[3..3]: print(x)
print(xs[2..2].len, xs[0..5].len, xs[0])
