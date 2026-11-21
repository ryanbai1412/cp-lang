// Codeforces 2138 E1: Determinant Construction (Easy Version) (dream).
// The tridiagonal matrix with diagonal d, 1 above and -1 below has determinant
// K(d), the continuant: K(d ++ [c]) = c * K(d) + K(d without its last entry).
// With entries in {0, 1} that recurrence is the subtractive Euclid walk run
// backwards, so x is reached from the pair (z, x - z) exactly when gcd(x, z) = 1,
// and z near x / phi keeps the walk short. `room` bounds the side length.

// The diagonal built by the walk from (a, b) down to (1, 1), if it fits in `room`.
fn walk(a: Int, b: Int, room: Int) -> ?List<0..=1> =
    if room == 0: none
    elif a < b: [0] ++ walk(b, a, room - 1)?
    elif a == b: if a == 1: [1] else: none
    else: [1] ++ walk(b, a - b, room - 1)?

@!multitest
let x: 0..=1e7 = read()
let diag = if x <= 1: [x] else:
    (1..x).nearest_first(to: x / phi).first_some(z => [1] ++ walk(z, x - z, room: 79)?)!
domain N = 0..diag.len
print(N.count)
print([N, N] (i, j) => match j - i: 0 => diag[i]; 1 => 1; -1 => -1; _ => 0)
