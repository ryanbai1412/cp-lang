// `desc: true` on `sort` and `sorted`, alone and after a key.
var xs = [3, 1, 2, 5]
xs.sort(desc: true)
let down = true
var a: [0..4]Int = [4, 9, 1, 7]
a.sort(desc: down)
print(xs, xs.sorted(desc: false), sorted(xs, desc: true), a)
var ws = ["bb", "a", "ccc"]
ws.sort(by: w => w.len, desc: true)
print(ws, sorted(ws, by: w => w.len, desc: not down), sorted(ws, desc: true, by: w => w.len))
sort(mut xs, desc: true)
print(xs)
