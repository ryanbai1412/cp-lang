@!multitest
let n: Int = read()
let len = n * n + 1
var lcp: [0..len]Int = n
var ans: List<Int> = []
for i in 0..=n:
    if i == n:
        var v = 0
        while lcp[v] != n: v += 1
        for j in (0..=n).rev:
            while lcp[v] != j: v -= 1
            ans.push(v + 1)
        ans.reverse()
        break
    let q = [v + 1 for v in 0..len if lcp[v] == n]
    print("?", q.len, q)
    flush()
    let cnt: Int = read()
    let got: [0..cnt]Int = read()
    for v in got: lcp[v - 1] = i
    if cnt > n:
        for v in 0..len:
            if lcp[v] == i and ans.len <= n:
                ans.push(v + 1)
        break
print("!", ans)
flush()
