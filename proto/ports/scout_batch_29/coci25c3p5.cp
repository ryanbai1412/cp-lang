// COCI 2025/26 Contest 3 P5: n fans of lattice points; fan i has the
// pair (a[i], b[i]) and b[i] + 1 points of its own, there are n + 1 base
// points, and neighbouring fans i - 1 and i share the points counted by
// `shared` below. Print the number of distinct points.
//
// Ported from the formula in the accepted submission, not from the
// statement, which was not at hand (see the README).
// original: corpus/submissions/7730259.cpp

let n: Int = read()
domain Fan = 1..=n
let a: [Fan]Int = read()
let b: [Fan]Int = read()

/// The points fans l and r have in common.
fn shared(l: Fan, r: Fan) -> Int:
    let (gl, gr) = (gcd(a[l], b[l]), gcd(a[r], b[r]))
    let step = lcm(a[l] / gl, a[r] / gr)
    let scale = gcd(b[l] / gl, b[r] / gr)
    min(a[l], a[r]) * scale / step + 1

print(n + 1 + sum(b[i] + 1 for i in Fan) - sum(shared(i.prev!, i) for i in Fan if i != Fan.first))
