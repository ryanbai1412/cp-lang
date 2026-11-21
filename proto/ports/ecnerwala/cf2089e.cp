use lib.math.poly.{Z, convolve, integral}

fn add(a: List<Z>, b: List<Z>) -> List<Z>:
    var sum = a.resized(max(a.len, b.len), 0)
    for (i, x) in b.items: sum[i] += x
    sum

fn remove(a: List<Z>, b: List<Z>) -> List<Z>:
    if a.is_empty: return []
    convolve(a, b.rev())[b.len - 1..].to_list()

fn plus_integral(a: List<Z>) -> List<Z> = add(a, integral(a))

@!multitest
let n: Int = read()
var parent: [1..=n]?Int = none
for _ in 1..n:
    let (u, v): (Int, Int) = read()
    parent[max(u, v)] = min(u, v)

var sub: [1..=n]List<Z> = [1]
for v in (1..=n).rev:
    sub[v] = plus_integral(sub[v])
    if let p = parent[v]: sub[p] = convolve(sub[p], sub[v])

var before: [1..=n]List<Z> = []
var siblings: [1..=n]List<Z> = [1]
for v in 2..=n:
    let p = parent[v]!
    before[v] = siblings[p]
    siblings[p] = convolve(siblings[p], sub[v])

var ans: List<Z> = [1]
var dp: [1..=n + 1]List<Z> = List.filled(n + 1, Z.zero)
dp[n + 1][1] = 1
for t in (2..=n).rev:
    dp[t] = add(dp[t], dp[t + 1])
    let stop = if t < n: parent[t + 1]! else: 1
    var ways: List<Z> = [1]
    var (p, child) = (t, t + 1)
    while p != stop:
        if p != t: ways = convolve(ways, before[child])
        dp[p] = add(dp[p], remove(dp[t + 1], integral(ways)))
        ways = plus_integral(ways)
        (p, child) = (parent[p]!, p)
    dp[t + 1] = []

    var carry: List<Z> = []
    var node: ?Int = t
    child = t + 1
    while let v = node:
        if v != t: carry = remove(carry, before[child])
        var lifted: List<Z> = [0]
        for (i, x) in dp[v].items: lifted.push(x * (i + 1))
        let next = add(lifted, carry)
        carry = add(carry, dp[v])
        dp[v] = next
        (node, child) = (parent[v], v)
    ans.push(dp[1][0])
print(ans)
