// cco16p4: two n x n red/white patterns are equivalent when toggles of 2x2 blocks turn one into
// the other. Sweeping the top-left (n-1)^2 cells to white leaves a canonical last row and column;
// count pairs with the same canonical form.
let (n, q) = read<(2..=10, 1..=100_000)>()
fn canon() -> Int:
    var g: [n, n]Bool = read<[n, n]Char>().map(c => c == 'R').collect()
    for i in 0..n - 1:
        for j in 0..n - 1:
            if g[i, j]:
                for (di, dj) in [(0, 0), (0, 1), (1, 0), (1, 1)]:
                    g[i + di, j + dj] ^= true
    chain(g[.., n - 1], g[n - 1, ..]).fold((m, b) => m * 2 + b.int, 0)
print([canon() for _ in 0..q].tally().values.map(c => c * (c - 1) / 2).sum())
