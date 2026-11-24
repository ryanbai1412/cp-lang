// Source: CCC 2023 S3 Palindromic Poster, https://dmoj.ca/problem/ccc23s3
// Problem: N, M in 2..=2000, R in 0..=N, C in 0..=M. Print an N x M grid of lowercase letters with
//   exactly R palindromic rows and C palindromic columns (any one), or IMPOSSIBLE.
// Solution: cases. 0 < R < N and 0 < C < M: all 'a' but 'b' in the bottom-right block below row
//   R and right of column C. R = 0 (C = 0 the transpose): an a/b/c/d frame for C = 0, else a 'b'
//   right column and 'c' in the bottom row from column C on. R = N (C = M the transpose): all
//   'a' when C = M; impossible if M is even and C odd; else a symmetric bottom row of 'b' with C
//   'a's from both ends and the middle. O(N M).
// original: corpus/submissions/5350806.cpp
// Bounds: n, m: 2..=2000; r: 0..=n; c: 0..=m
let n: 2..=2000 = read()
let m: 2..=2000 = read()
let r: 0..=n = read()
let c: 0..=m = read()
domain Row = 0..n
domain Col = 0..m

if r > 0 and r < n and c > 0 and c < m:
    print([Row, Col] (i, j) => if i.ord >= r and j.ord >= c: 'b' else: 'a')
else:
    // Every case left has all rows or none palindromic, or else all or no columns: transpose the
    // second kind so the rows are the extreme side. lines x len poster, `pal` palindromic lines
    // across (0 or `lines`), `cross` palindromic lines the other way.
    let flip = r != 0 and (c == 0 or r != n)
    let (lines, len, pal, cross) = if flip: (m, n, c, r) else: (n, m, r, c)
    // The letter at line i, position j.
    fn letter(i: Int, j: Int) -> Char:
        let bottom = i == lines - 1
        if pal == 0:
            // The last position of every line is 'b' and the bottom line is 'c' from `cross` on,
            // so no line reads the same both ways; with cross = 0 the corner a/b/c/d frame does it.
            if cross == 0 and bottom: return if j == len - 1: 'd' elif j == 0: 'c' else: 'b'
            if bottom and j >= cross: return 'c'
            if j == len - 1: 'b' else: 'a'
        else:
            // All 'a', and the bottom line is a palindrome of 'b' with `cross` 'a's from both ends
            // and the middle, so exactly `cross` of the crossing lines stay palindromes.
            let ends = cross / 2
            let keep = j < ends or j >= len - ends or (cross % 2 == 1 and j == len / 2)
            if cross < len and bottom and not keep: 'b' else: 'a'
    if pal == lines and len % 2 == 0 and cross % 2 == 1: print("IMPOSSIBLE")
    else: print([Row, Col] (i, j) => if flip: letter(j.ord, i.ord) else: letter(i.ord, j.ord))
