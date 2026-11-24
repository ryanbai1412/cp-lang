// A descending key on `sort` and `sorted`, alone and around another key.
var xs = [3, 1, 2, 5]
xs.sort(by: x => desc(x))
let down = true
var a: [0..4]Int = [4, 9, 1, 7]
if down: a.sort(by: x => desc(x)) else: a.sort()
print(xs, xs.sorted(), sorted(xs, by: x => desc(x)), a)
var ws = ["bb", "a", "ccc"]
ws.sort(by: w => desc(w.len))
print(ws, (if not down: sorted(ws, by: w => desc(w.len)) else: sorted(ws, by: w => w.len)), sorted(ws, by: w => desc(w.len)))
sort(mut xs, by: x => desc(x))
print(xs)
