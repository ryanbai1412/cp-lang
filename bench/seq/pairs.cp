// ioi25_triples' small-group loop: every pair of each group of equal keys, through a nested
// fn that writes the fn's count.
fn solve() -> Int:
    let n = 600_000
    var seed = 123456789
    var h = List<Int>()
    for _ in 0..n:
        seed = (seed * 1103515245 + 12345) % 2147483647
        h.push(seed % 200_000)
    var found = 0
    fn crossed(i: Int, j: Int, k: Int):
        if k < n and (h[i] + h[j]) % 3 == h[k] % 3: found += 1
    let groups = (0..n).group(by: x => h[x])
    for _ in 0..40:
        for (_, group) in groups:
            for (i, j) in group.pairs(): crossed(i, j, i + h[j])
    found

print(solve())
