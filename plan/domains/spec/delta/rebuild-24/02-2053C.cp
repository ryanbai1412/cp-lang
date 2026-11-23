// CF 2053C Bewitching Stargazer. Every segment observed at one depth has the same length,
// so lucky(len) on [1, len] gives (sum, cnt) of aimed stars; the same length starting
// at l adds sum + cnt * (l - 1). One call per depth: O(log n).
fn lucky(len: Int, k: Int) -> (sum: Int, cnt: Int):
    if len < k: return (0, 0)
    let m = (1 + len) / 2
    if len % 2 == 0:
        let h = lucky(m, k)
        return (2 * h.sum + h.cnt * m, 2 * h.cnt)
    let h = lucky(m - 1, k)
    (m + 2 * h.sum + h.cnt * m, 2 * h.cnt + 1)

@!multitest
let (n, k): (1..=2e9, 1..=n) = read()
print(lucky(n, k).sum)
