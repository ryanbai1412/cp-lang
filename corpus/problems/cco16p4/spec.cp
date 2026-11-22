// cco16p4: two n x n red/white patterns are equivalent when toggles of 2x2 blocks turn one into
// the other. Sweeping every cell off the last row and column to white leaves a canonical last
// row and column; count pairs with the same canonical form.
let (n, q) = read<(2..=10, 1..=100_000)>()
domain N = 1..=n
fn canon() -> Int:
    var g: [N, N]Bool = read<[N, N]Char>().map(c => c == 'R')
    for i in ..N.last!:
        for j in ..N.last!:
            if g[i, j]:
                for (a, b) in [(i, j), (i, j.next!), (i.next!, j), (i.next!, j.next!)]:
                    g[a, b] ^= true
    chain(g[.., N.last!], g[N.last!, ..]).fold((m, b) => m * 2 + Int(b), 0)
print([canon() for _ in 0..q].tally().values.map(c => c * (c - 1) / 2).sum())
