// Codeforces 2138 E2: Determinant Construction (Hard Version) (dream).
// The tridiagonal matrix with diagonal d, 1 above and -1 below has determinant
// K(d), the continuant: K(d + [c]) = c * K(d) + K(d without its last entry).
// With entries in {0, 1} that recurrence is the subtractive Euclid walk run
// backwards, so x is reached from the pair (z, x - z) exactly when gcd(x, z) = 1,
// and z near x / phi keeps the walk short. Trying z outward from x / phi finds a
// walk within MAX_N = 50 for every x <= 1e7 (checked); after prepending [1],
// the walk has MAX_N - 1 room.

const MAX_N = 50

// The diagonal built by the walk from (a, b) down to (1, 1), if it fits in `room`.
fn walk(a: Int, b: Int, room: Int) -> ?List<0..=1> =
    if room == 0: None
    elif a < b: [0] + walk(b, a, room - 1)?
    elif a == b: if a == 1: [1] else: None
    else: [1] + walk(b, a - b, room - 1)?

let phi = (1.0 + sqrt(5.0)) / 2.0

@!multitest
let x: 0..=1e7 = read()
let diag = if x <= 1: [x] else:
    (1..x).nearest_first(to: x / phi).first_some(z => walk(z, x - z, room: MAX_N - 1).map(w => [1] + w))!
domain N = 0..diag.len
print(N.count)
print([N, N] (i, j) =>
    match j - i:
        0 => diag[i]
        1 => 1
        -1 => -1
        _ => 0
)
