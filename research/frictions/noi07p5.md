# Friction report: noi07p5

## Dream

The port is the dream: Kirchhoff via `Mat<Z>.det()` and `guess_at` for the recurrence.

## Hand determinant becomes `Mat.det`

The v0.2 port wrote its own row reduction over `List<List<Z>>`. The rewrite builds the
reduced Laplacian with `Mat<Z>.from_rows` and calls `det()`. `k` and `n` are bounded in the
read type.

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++. Corpus tests pass.
