// CCC 2023 S3, Palindromic Poster — ideal CP Lang sketch
// Category: Multi-answer Constructive + Custom Checker (char grid + IMPOSSIBLE sentinel)
// Original C++: corpus/submissions/5350806.cpp (193 LOC raw, ~140 LOC stripped incl. built-in grade())
// Buildable today-form: proto/ports/scout_batch_19/today/ccc23s3.cp (verified exhaustively n,m in 2..6)

let n: 2..=2000 = read()
let m: 2..=2000 = read()
let r: 0..=n = read()
let c: 0..=m = read()
domain Row = 0..n
domain Col = 0..m

fn construct(n: Int, m: Int, r: Int, c: Int) -> ?[Row, Col]Char:
    // Note: in ideal CP Lang, a transposed view avoids allocating or threading a `flip` bool!
    var g: [Row, Col]Char = 'a'
    if 0 < r < n and 0 < c < m:
        for i in Row if i.label >= r:
            for j in Col if j.label >= c:
                g[i, j] = 'b'
        return g
    if r == 0:
        if c == 0:
            for i in 0..n: g[Row(i), Col.last] = 'b'
            for j in 0..m: g[Row.last, Col(j)] = 'b'
            g[Row.last, Col.first] = 'c'
            g[Row.last, Col.last] = 'd'
        else:
            for i in Row: g[i, Col.last] = 'b'
            for j in c..m: g[Row.last, Col(j)] = 'c'
        return g
    if c == 0:
        return construct(m, n, c, r)?.transposed()
    if r == n:
        if c == m: return g
        if m % 2 == 0 and c % 2 == 1: return none
        for j in Col: g[Row.last, j] = 'b'
        if c % 2 == 1: g[Row.last, Col(m / 2)] = 'a'
        for k in 0..(c / 2):
            g[Row.last, Col(k)] = 'a'
            g[Row.last, Col(m - 1 - k)] = 'a'
        return g
    construct(m, n, c, r)?.transposed()

// WISH B19-6: optional output sentinel `none: "IMPOSSIBLE"` symmetric with `read(none: ...)`
print(construct(n, m, r, c), none: "IMPOSSIBLE")

// WISH B19-7: co-located `checker:` reusing input bindings (`n, m, r, c`) and typed output reading,
// plus `feasible(...)` / `spec` cross-check so returning IMPOSSIBLE on a solvable case is caught!
checker:
    let out: ?[Row, Col]Char = read_output(none: "IMPOSSIBLE")
    let solvable = not ((r == n and m % 2 == 0 and c % 2 == 1)
                     or (c == m and n % 2 == 0 and r % 2 == 1))
    match out:
        none =>
            require not solvable, "output IMPOSSIBLE on a solvable case"
        some(g) =>
            require solvable, "output a grid on an impossible case"
            require all('a' <= g[i, j] <= 'z' for i in Row for j in Col)
            let pal_rows = count(i in Row: all(g[i, j] == g[i, Col.last - j.ord] for j in Col))
            let pal_cols = count(j in Col: all(g[i, j] == g[Row.last - i.ord, j] for i in Row))
            require pal_rows == r and pal_cols == c
