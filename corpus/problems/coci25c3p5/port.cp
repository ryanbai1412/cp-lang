// Source: COCI 2025/26 Round 3 P5 (Ravnalo),
//   https://hsin.hr/coci/archive/2025_2026/contest3_tasks.pdf
// limits: 1000 ms, 512 MB
// Problem: n <= 1e5 width-1 columns side by side; column i has height a_i and is cut into
//   b_i equal parts (a_i, b_i <= 1e9). Print the fewest straight segments that draw every
//   column edge and every division line.
// Solution: n + 1 vertical segments; column i has b_i + 1 horizontal lines at heights
//   k a_i / b_i, and a height shared with the next column extends into one segment. With
//   a/b = p/q in lowest terms, two columns share the multiples of lcm(p_l, p_r) /
//   gcd(q_l, q_r) up to min(a_l, a_r); subtract that count per adjacent pair. O(n log).
// original: corpus/submissions/7730259.cpp
// Bounds: n: 1..=1e5; a, b: 1..=1e9

let n: 1..=1e5 = read()
domain Fan = 1..=n
let a: [Fan]1..=1e9 = read()
let b: [Fan]1..=1e9 = read()

// the heights where fans l and r both have a line
fn shared(l: Fan, r: Fan) -> Int:
    let (gl, gr) = (gcd(a[l], b[l]), gcd(a[r], b[r]))
    let step = lcm(a[l] / gl, a[r] / gr)
    let scale = gcd(b[l] / gl, b[r] / gr)
    min(a[l], a[r]) * scale / step + 1

print(n + 1 + sum(b[i] + 1 for i in Fan) - sum(shared(l, r) for r in Fan if let l = r.prev))
