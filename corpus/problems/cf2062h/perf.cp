// Codeforces 2062H (Galaxy Generator), tuned.
// port.cp's dp, with `ways` and `galaxies` each one flat table indexed
// `(hi * (n + 1) + lo) * 2^n + cols`, as orig.cpp keeps them, the star rows and
// column sets as plain bit masks, and the hot loops `unchecked:`.
type Z = Mod<1_000_000_007>
@!multitest
let n: 1..=14 = read()
let grid: [n]Str = read()
let full = 1 << n
let w1 = n + 1
var stars: [0..n]Int = 0
for r in 0..n:
    for c in 0..n:
        if grid[r][c] == '1': stars[r] |= 1 << c
fn at(hi: Int, lo: Int) -> Int = (hi * w1 + lo) * full
var ways: [0..w1 * w1 * full]Z = 0
var galaxies: [0..w1 * w1 * full]Z = 0
for hi in 0..=n:
    let hh = at(hi, hi)
    unchecked:
        for c in 0..full: ways[hh + c] = 1
    for mid in (0..hi).rev:
        let (cur, prev) = (at(hi, mid), at(hi - 1, mid))
        unchecked:
            for c in 0..full:
                ways[cur + c] += ways[prev + c]
                galaxies[cur + c] += galaxies[prev + c]
        for y_lo in (0..n).rev:
            var band = 0
            for y in y_lo..n:
                band |= 1 << y
                let inside = sum(popcount(stars[r] & band) for r in mid..hi)
                let one_box = Z(2) ** inside - ways[cur + band]
                let other = (full - 1) & ~band
                for lo in 0..=mid:
                    let (dst, src) = (at(hi, lo), at(mid, lo))
                    var rest = other
                    while true:
                        unchecked:
                            let w = ways[src + rest]
                            let g = galaxies[src + rest]
                            ways[dst + (rest | band)] += w * one_box
                            galaxies[dst + (rest | band)] += (w + g) * one_box
                        if rest == 0: break
                        rest = (rest - 1) & other
print(galaxies[at(n, 0) + full - 1])
