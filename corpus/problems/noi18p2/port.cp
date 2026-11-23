// NOI '18 P2 - Inverse (runnable on proto/cp.py lib-run today)

type Z = Mod<998244353>

let t: Int = read()
var cases: List<List<Int>> = []
var max_n = 1
for _ in 0..t:
    let n: Int = read()
    if n > max_n: max_n = n
    var q: List<Int> = []
    for _ in 0..n:
        let v: Int = read()
        q.push(v)
    cases.push(q)

let comb = Comb<Z>(2 * max_n + 5)

fn safe_choose(n: Int, k: Int) -> Z:
    if n < 0 or k < 0 or k > n: return 0
    comb.choose(n, k)

fn ballot(n: Int, i: Int, low: Int) -> Z:
    if low > n: return 0
    let steps = 2 * n - i - low
    safe_choose(steps, n - i) - safe_choose(steps, n - low - 2)

for c in 0..t:
    let q = cases[c]
    let n = q.len
    var seen = List.filled(n + 2, false)
    var min_unused = 1
    var mx = 0
    var ans: Z = 0

    for i in 1..=n:
        let v = q[i - 1]
        let low = max(mx, v) + 1
        ans += ballot(n, i, low)

        if v > mx:
            mx = v
        elif v != min_unused:
            break

        seen[v] = true
        while min_unused <= n and seen[min_unused]:
            min_unused += 1

    print(ans)
