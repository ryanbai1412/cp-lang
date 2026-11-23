// CF 2052J Judicious Watching. Tasks go in deadline order, each started as late as the
// later ones allow (latest[j]); watch the next episode whenever it ends by the next
// task's latest start, else do that task. finish[e] is when episode e ends; queries are
// answered offline in time order with one pointer.
@!multitest
let n, m, q: 1..=2e5 = read()
let a = read<[n]1..=1e9>()
let d = read<[n]1..=1e15>()
let l = read<[m]1..=1e9>()
let t = read<[q]1..=1e15>()

let order = (0..n).sorted(by: i => d[i])
var latest: [n]Int = 0
var next_start: ?Int = None
for j in (0..n).rev:
    let i = order[j]
    latest[j] = min(d[i], next_start ?? d[i]) - a[i]
    next_start = latest[j]

var finish: List<Int> = []
var cur = 0
var j = 0
for e in 0..m:
    while j < n and cur + l[e] > latest[j]:
        cur += a[order[j]]
        j += 1
    cur += l[e]
    finish.push(cur)

var ans: [q]Int = 0
var watched = 0
for k in (0..q).sorted(by: k => t[k]):
    while watched < m and finish[watched] <= t[k]: watched += 1
    ans[k] = watched
print(ans)
