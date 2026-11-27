// expect: ok, output "0 2 7\n6 20 9\n2 0"
// rule: R04b.1.5 R04b.1.6 R04.3.4
// `[r]T` for a range `r` of `Int`s covers just `r`'s keys, and a table over an anonymous range
// walks its keys as `Int`s, so `[w.keys] k => w[k]` keeps a view's keys.
let s = [5, 6, 7, 8]
let w = s[2..]
let t: [w.keys]Int = 0
let u = [w.keys] k => w[k]
print(t[2], t.len, u[2])
fn doubled(n: Int) -> Int:
    let v = [0..n] k => s[k] * 2
    v[0] + v[n - 1] - 4
let a = [0..4] k => s[k] + 1
let b = [1..=3] k => k * 3
print(a[0], doubled(3), b[3])
var c: [s.keys]Int = 0
for k in s.keys: c[k] += k % 2
print(c[1] + c[3], c[0])
