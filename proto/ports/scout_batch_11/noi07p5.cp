// NOI 2007 P5, Counting Spanning Trees: vertices 1..=n in a row, an edge
// between every two at distance at most k (k <= 5, n <= 1e15). The number
// of spanning trees modulo 65521.
// The original spends 60 lines on a bitmask DP for the determinant, 30 on
// Berlekamp-Massey and 40 on a matrix power. Here the small cases are
// Kirchhoff's theorem as written, and `guess_at` finds and applies the
// linear recurrence they satisfy.
// original: corpus/submissions/3883675.cpp
use std.spec.{guess_at}

const P = 65521
type Z = Mod<P>
type Mat = List<List<Z>>

// Row reduction to an upper triangle; a row swap flips the sign.
fn det(m: Mat) -> Z:
    var (a, d) = (m, Z(1))
    let n = a.len
    for c in 0..n:
        var pivot = c
        while pivot < n and a[pivot][c] == 0: pivot += 1
        if pivot == n: return 0
        if pivot != c:
            swap(a[pivot], a[c])
            d = -d
        d *= a[c][c]
        let inv = a[c][c].inv
        for r in c + 1..n:
            if a[r][c] != 0:
                let f = a[r][c] * inv
                for j in c..n: a[r][j] -= f * a[c][j]
    d

let (k, n): (Int, Int) = read()

// Kirchhoff: the Laplacian (degree on the diagonal, -1 per edge) with one vertex removed.
fn trees(size: Int) -> Z:
    det([[Z(if i == j: min(k, i) + min(k, size - 1 - i) else: if abs(i - j) <= k: -1 else: 0)
          for j in 0..size - 1] for i in 0..size - 1])

// terms[i] is the answer for k + i vertices; more terms until the recurrence is confirmed
var terms = List<Z>()
while true:
    terms.push(trees(k + terms.len))
    if n - k < terms.len:
        print(terms[n - k])
        break
    if let answer = guess_at(terms, n - k):
        print(answer)
        break
