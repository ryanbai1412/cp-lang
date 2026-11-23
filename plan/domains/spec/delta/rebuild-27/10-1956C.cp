// CF 1956C: write rows then columns n..=1 with the identity permutation;
// cell (i, j) ends as max(i, j), the maximum possible, in 2n operations.
@check fn valid() -> Bool:
    let n: 1..=500 = read()
    let s: Int = read_output()
    let ops: 0..=2 * n = read_output()
    var cell: [1..=n, 1..=n]Int = 0
    for _ in 0..ops:
        let kind: 1..=2 = read_output()
        let at: 1..=n = read_output()
        let perm: [1..=n]1..=n = read_output()
        var seen: [1..=n]Bool = false
        for j in 1..=n:
            if seen[perm[j]]: return false
            seen[perm[j]] = true
            if kind == 1: cell[at, j] = perm[j]
            else: cell[j, at] = perm[j]
    let best = (1..=n).map(i => i * (2 * i - 1)).sum()
    s == best and cell.sum() == best

@!multitest
let n: 1..=500 = read()
print((1..=n).map(i => i * (2 * i - 1)).sum(), 2 * n)
for i in (1..=n).rev:
    print(1, i, (1..=n).collect())
    print(2, i, (1..=n).collect())
