// CCC 2023 S3, Palindromic Poster (multi-answer constructive, buildable today-form)
// original: corpus/submissions/5350806.cpp
let n: 2..=2000 = read()
let m: 2..=2000 = read()
let r: 0..=n = read()
let c: 0..=m = read()
domain Row = 0..n
domain Col = 0..m

var grid: [Row, Col]Char = 'a'

fn put(mut g: [Row, Col]Char, flip: Bool, i: Int, j: Int, ch: Char):
    if flip:
        g[Row(j), Col(i)] = ch
    else:
        g[Row(i), Col(j)] = ch

fn fill_zero_rows(mut g: [Row, Col]Char, flip: Bool, nn: Int, mm: Int, cc: Int):
    if cc == 0:
        put(mut g, flip, 0, 0, 'a')
        put(mut g, flip, 0, mm - 1, 'b')
        put(mut g, flip, nn - 1, 0, 'c')
        put(mut g, flip, nn - 1, mm - 1, 'd')
        for i in 1..(nn - 1):
            put(mut g, flip, i, 0, 'a')
            put(mut g, flip, i, mm - 1, 'b')
        for j in 1..(mm - 1):
            put(mut g, flip, 0, j, 'a')
            put(mut g, flip, nn - 1, j, 'b')
    else:
        for i in 0..nn:
            put(mut g, flip, i, mm - 1, 'b')
        if cc < mm:
            for j in cc..mm:
                put(mut g, flip, nn - 1, j, 'c')

fn fill_all_rows(mut g: [Row, Col]Char, flip: Bool, nn: Int, mm: Int, cc: Int) -> Bool:
    if cc == mm:
        return true
    if mm % 2 == 0 and cc % 2 == 1:
        return false
    for j in 0..mm:
        put(mut g, flip, nn - 1, j, 'b')
    var rem = cc
    if rem % 2 == 1:
        put(mut g, flip, nn - 1, mm / 2, 'a')
        rem -= 1
    var cl = 0
    var cr = mm - 1
    while rem > 0:
        put(mut g, flip, nn - 1, cl, 'a')
        put(mut g, flip, nn - 1, cr, 'a')
        cl += 1
        cr -= 1
        rem -= 2
    true

fn solve():
    var ok = true
    if r > 0 and r < n and c > 0 and c < m:
        for i in Row:
            for j in Col:
                if i.label >= r and j.label >= c:
                    grid[i, j] = 'b'
    elif r == 0:
        fill_zero_rows(mut grid, false, n, m, c)
    elif c == 0:
        fill_zero_rows(mut grid, true, m, n, r)
    elif r == n:
        ok = fill_all_rows(mut grid, false, n, m, c)
    else:
        ok = fill_all_rows(mut grid, true, m, n, r)
    if not ok:
        print("IMPOSSIBLE")
    else:
        print(grid)

solve()
