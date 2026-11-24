// A map entry whose key calls a fn is lent when the call can't change the
// map, and copied when it can.
fn key(r: Int) -> Int = r % 3
let at = (0..7).group(by: k => k % 3)
var s = 0
for r in 0..7:
    let xs = at[key(r)]
    s += xs[xs.len - 1]
print(s)
var m = Map<Int, List<Int>>()
fn grow(r: Int) -> Int:
    m[r + 100] = [r]
    r % 3
for r in 0..3: m[r] = [r, r]
var t = 0
for r in 0..3:
    let ys = m[grow(r)]
    t += ys.len + ys[0]
print(t, m.len)
