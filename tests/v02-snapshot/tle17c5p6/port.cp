// Source: TLE '17 Contest 5 P6 (Circuits), https://dmoj.ca/problem/tle17c5p6
// Problem: n <= 1e5 inputs, all an unknown x, feed m <= 2e5 NAND gates, each reading two lower
//   gates; gate n + m is the output. Fix as many inputs as possible to 0 or 1 so the output still
//   matches the unfixed circuit for both x = 0 and x = 1; print a string over 0, 1 and x.
// Solution: light(k), the output with inputs 1..=k on and the rest off, is one memoised
//   evaluation, O(n + m). If light(0) = light(n) the output ignores x and all zeros works.
//   Otherwise a binary search (first_true) finds k with light(k - 1) = light(0) != light(k); print
//   k - 1 ones, x, then zeros. O((n + m) log n).
// original: corpus/submissions/5142110.cpp
// note: as it builds today, "the last count that still works" is a first_true on the
// negation with a `- 1` (R1).
let n: 1..=1e5 = read()
let m: 1..=2e5 = read()
domain Wire = 1..=n + m
domain Gate = 1..=m

// every gate reads two earlier wires
fn random_gates() -> [Gate](a: Wire, b: Wire):
    [Gate] g => (Wire(rand(1, n + g.label - 1)), Wire(rand(1, n + g.label - 1)))

let gates: [Gate](a: Wire, b: Wire) = read(gen: random_gates)

// the output of the last gate when the first `ones` inputs are on and the rest off
var value: [Wire]?Bool = None
fn out(w: Wire) -> Bool:
    if let v = value[w]: return v
    let g = gates[Gate(w.label - n)]
    let v = not (out(g.a) and out(g.b))
    value[w] = v
    v
fn light(ones: Int) -> Bool:
    for w in Wire:
        if w.label <= n: value[w] = w.label <= ones
        else: value[w] = None
    out(Wire.last!)

// If all off and all on differ, some input flips the light when it is the
// only one between a block of ones and a block of zeros.
let dark = light(0)
var word: [n]Char = '0'
if dark != light(n):
    let flip = (0..=n).first_true(k => light(k) != dark)! - 1
    for i in 0..flip: word[i] = '1'
    word[flip] = 'x'
print(word)
