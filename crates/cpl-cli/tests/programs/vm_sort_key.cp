let n: Int = read()
var xs: List<Int> = [i * 37 % 11 for i in 0..n]
xs.sort(by: x => x % 3)
print(xs)
if n > 5: xs.sort(by: x => desc(-x)) else: xs.sort(by: x => -x)
print(xs)
var ws = ["pear", "fig", "banana", "kiwi", "plum", "apple"]
ws.sort(by: w => desc(w.len))
print(ws)
ws.sort(by: w => (w.len, w))
print(ws)
var ps: List<(Int, Str)> = [(3, "c"), (1, "a"), (3, "a"), (2, "b")]
ps.sort(by: p => p.0)
print(ps)
var arr: [0..5]Int = [5, 3, 9, 1, 7]
arr.sort(by: x => -x)
print(arr)
var rows: List<List<Int>> = [[3, 1], [1, 2], [2, 0]]
rows[0].sort(by: x => -x)
rows.sort(by: r => r[1])
print(rows)
var empty: List<Int> = []
empty.sort(by: x => x)
print(empty.len)
var groups: Map<Int, List<Int>> = []
var hgroups: HashMap<Int, List<Int>> = []
for g in 0..2:
    groups[g] = []
    hgroups[g] = []
for i in 0..n:
    groups[i % 2].push(i * 5 % 7)
    hgroups[i % 2].push(i * 3 % 8)
groups[0].sort()
groups[1].sort(by: x => -x)
hgroups[1].sort(by: x => desc(x))
hgroups[0].sort(by: x => desc(x % 4))
print(groups[0], groups[1], hgroups[0], hgroups[1])
fn mod5(x: Int) -> Int: x % 5
fn by_name(p: (Int, Str)) -> (Str, Int): (p.1, -p.0)
var ys = [i * 7 % 13 for i in 0..n]
ys.sort(by: mod5)
print(ys)
let last = (x: Int) => x % 10
ys.sort(by: x => desc(last(x)))
print(ys)
ps.sort(by: by_name)
print(ps)
var arr2: [0..5]Int = [4, 8, 15, 16, 23]
arr2.sort(by: mod5)
print(arr2)
