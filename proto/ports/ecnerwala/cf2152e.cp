@!multitest
let n: Int = read()
let len = n * n + 1
var lcp: [0..len]Int = n
for i in 0..n:
    let q = [v + 1 for v in 0..len if lcp[v] == n]
    print("?", q.len, q)
    flush()
    let cnt: Int = read()
    let got: [0..cnt]Int = read()
    for v in got: lcp[v - 1] = i
    if cnt > n:
        print("!", [v + 1 for v in 0..len if lcp[v] == i][..=n])
        flush()
        return

var v = first(u for u in 0..len if lcp[u] == n)!
var ans: List<Int> = []
for j in (0..=n).rev:
    while lcp[v] != j: v -= 1
    ans.push(v + 1)
ans.reverse()
print("!", ans)
flush()
