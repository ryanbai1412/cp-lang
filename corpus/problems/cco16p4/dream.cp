# CCO 2016 P4 Colourful Squares-ish: an n×n grid of R/B; a move flips the four cells of any 2×2
# square. Count pairs of the q grids that can reach each other.
# Clearing the top-left (n-1)×(n-1) block greedily is forced, so the last row and column left
# behind are a canonical form; equal forms mean reachable.
let (n, q) = read (Int, Int)
domain R = [n]
domain C = [n]
fn canon(var g: [R, C]Bool) -> (List<Bool>, List<Bool>):
    for (r, c) in (R.but_last, C.but_last) if g[r, c]:
        for (dr, dc) in [(0, 0), (0, 1), (1, 0), (1, 1)]: g[r + dr, c + dc] ^= true
    (R.map(r => g[r, C.last]), C.map(c => g[R.last, c]))
let forms = Counter((for _ in 0..q: canon(read [R, C]Char.map(ch => ch == 'R'))))
print(forms.values.sum(of: k => k * (k - 1) / 2))
