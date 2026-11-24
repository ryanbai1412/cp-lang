// CCO 2016 P4 Colourful Squares-ish: an n×n grid of R/B; a move flips the four cells of any 2×2
// square. Count pairs of the grids that can reach each other.
// Clearing the top-left (n-1)×(n-1) block greedily is forced, so the last row and column left
// behind are a canonical form; equal forms mean reachable.
let n: 2..=10 = read()
let g: 2..=10_000 = read()
domain R = 0..n
domain C = 0..n

fn canon(grid: [R, C]Char) -> (List<Bool>, List<Bool>):
    var red = [R, C] (r, c) => grid[r, c] == 'R'
    for r in 0..n - 1:
        for c in 0..n - 1:
            if red[R(r), C(c)]:
                for (dr, dc) in [(0, 0), (0, 1), (1, 0), (1, 1)]: red[R(r + dr), C(c + dc)] ^= true
    ([red[r, C.last!] for r in R], [red[R.last!, c] for c in C])

let forms = Counter([canon(read<[R, C]Char>()) for _ in 0..g])
print(forms.values().sum(of: k => k * (k - 1) / 2))
