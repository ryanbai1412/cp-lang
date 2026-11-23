// workaround for B2: negated key instead of desc(...) in Heap(by:)
// CF1942D Learning to Paint. dp[i]: the k best beauties of cells before i
// with cell i unpainted (cells 0 and n + 1 are virtual). dp[i] merges
// dp[j] + a[j+1][i-1] over all j < i with a heap holding one cursor per j.
@!multitest
let n: 1..=1000 = read()
let k: 1..=5000 = read()
domain V = 1..=n
var a: [V, V]Int = 0
for i in V:
    for j in i..=V.last!:
        a[i, j] = read<-1e6..=1e6>()
let w = (j: Int, i: Int) => if j + 1 <= i - 1: a[V(j + 1), V(i - 1)] else: 0
var dp = [[0]]
for i in 1..=n + 1:
    var h: Heap<(v: Int, j: Int, t: Int)> = Heap(by: x => -x.v)
    for j in 0..i:
        h.push((v: dp[j][0] + w(j, i), j: j, t: 0))
    var row = List<Int>()
    while row.len < k and not h.is_empty:
        let top = h.pop()!
        row.push(top.v)
        if top.t + 1 < dp[top.j].len:
            h.push((v: dp[top.j][top.t + 1] + w(top.j, i), j: top.j, t: top.t + 1))
    dp.push(row)
print(dp[n + 1])
