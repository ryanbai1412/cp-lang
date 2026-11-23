// Source: NOI 2007 P5 (Counting Spanning Trees), https://dmoj.ca/problem/noi07p5
// Problem: vertices 1..=n in a row, an edge between every two at distance at most k
//   (2 <= k <= 5, k <= n <= 1e15). Print the number of spanning trees modulo 65521.
// Solution: the counts for k, k + 1, ... vertices satisfy a linear recurrence of bounded
//   order (a transfer over the last k vertices), so compute small cases with Kirchhoff's
//   theorem (Laplacian minus a row and column, determinant by elimination mod the prime,
//   O(size^3)) and let `guess_at` find the recurrence and evaluate it at n, adding terms
//   until it is confirmed. The original instead hand-writes the transfer DP, Berlekamp-Massey
//   and a matrix power.
// original: corpus/submissions/3883675.cpp
use std.spec.{guess_at}

const P = 65521
type Z = Mod<P>
type Mat = List<List<Z>>

// Row reduction to an upper triangle; a row swap flips the sign.
fn det(m: Mat) -> Z:
    var (a, d) = (m, Z.one)
    let n = a.len
    for c in 0..n:
        var pivot = c
        while pivot < n and a[pivot][c] == 0: pivot += 1
        if pivot == n: return 0
        if pivot != c:
            swap(mut a[pivot], mut a[c])
            d = -d
        d *= a[c][c]
        let inv = a[c][c].inv()
        for r in c + 1..n:
            if a[r][c] != 0:
                let f = a[r][c] * inv
                for j in c..n: a[r][j] -= f * a[c][j]
    d

let (k, n): (Int, Int) = read()

// Kirchhoff: the Laplacian (degree on the diagonal, -1 per edge) with one vertex removed.
fn trees(size: Int) -> Z:
    det(
        [
            [
                Z(
                    if i == j: min(k, i) + min(k, size - 1 - i) else: if abs(i - j) <= k: -1 else: 0
                ) for j in 0..size - 1
            ] for i in 0..size - 1
        ]
    )

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
