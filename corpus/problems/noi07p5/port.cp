// Source: NOI 2007 P5 (Counting Spanning Trees), https://dmoj.ca/problem/noi07p5
// limits: 600 ms, 256 MB
// Problem: vertices 1..=n in a row, an edge between every two at distance at most k
//   (2 <= k <= 5, k <= n <= 1e15). Print the number of spanning trees modulo 65521.
// Solution: the counts for k, k + 1, ... vertices satisfy a linear recurrence of bounded
//   order (a transfer over the last k vertices), so compute small cases with Kirchhoff's
//   theorem (the Laplacian minus a row and column, std `Mat.det` mod the prime, O(size^3))
//   and let `guess_at` find the recurrence and evaluate it at n, adding terms until it is
//   confirmed. The original instead hand-writes the transfer DP, Berlekamp-Massey and a
//   matrix power.
// original: corpus/submissions/3883675.cpp
// Bounds: k: 2..=5; n: 2..=1e15
use std.spec.{guess_at}

type Z = Mod<65521>
let (k, n): (2..=5, 2..=1e15) = read()

// Kirchhoff: the Laplacian of `size` vertices (degree on the diagonal, -1 per edge) with the
// last vertex removed
fn trees(size: Int) -> Z:
    fn entry(i: Int, j: Int) -> Z =
        if i == j: Z(min(k, i) + min(k, size - 1 - i)) elif abs(i - j) <= k: Z(-1) else: Z(0)
    Mat<Z>.from_rows([[entry(i, j) for j in 0..size - 1] for i in 0..size - 1]).det()

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
