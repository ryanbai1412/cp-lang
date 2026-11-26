// expect: ok, output "3 4\n2 7\n3 0\n6 7"
// rule: R07.4.6
let xs = [5, 6, 7, 8]
print(xs[1..].len, xs[..].len)
let t = xs[..=1]
print(t.len, xs[2..][2])
print(xs[..3].len, xs[4..].len)
let u = xs[1..][..=2]
print(u.nth(0), u[2])
