// Source: TLE '17 Contest 5 P6 (Circuits), https://dmoj.ca/problem/tle17c5p6
// Problem: n <= 1e5 inputs, all an unknown x, feed m <= 2e5 NAND gates, each reading two lower
//   gates; gate n + m is the output. Fix as many inputs as possible to 0 or 1 so the output still
//   matches the unfixed circuit for both x = 0 and x = 1; print a string over 0, 1 and x.
// Solution: light(k), the output with inputs 1..=k on and the rest off, is one memoised
//   evaluation, O(n + m). If light(0) = light(n) the output ignores x and all zeros works.
//   Otherwise a binary search (last_true) finds the last k with light(k) = light(0), so
//   light(k + 1) differs; print k ones, x, then zeros. O((n + m) log n).
// original: corpus/submissions/5142110.cpp
// Bounds: n: 1..=1e5; m: 1..=2e5; gates.a, gates.b: 1..=n + m
let n: 1..=1e5 = read()
let m: 1..=2e5 = read()
domain Wire = 1..=n + m
domain Gate = 1..=m
let gates: [Gate](a: Wire, b: Wire) = read()

var value: [Wire]?Bool = None
fn out(w: Wire) -> Bool:
    if let v = value[w]: return v
    let g = gates[Gate(w.label - n)]
    let v = not (out(g.a) and out(g.b))
    value[w] = v
    v

// the circuit's output with the first `ones` inputs on and the rest off
fn light(ones: 0..=n) -> Bool:
    for w in Wire: value[w] = if w.label <= n: Some(w.label <= ones) else: None
    out(Wire.last!)

let dark = light(0)
var word: [n]Char = '0'
if dark != light(n):
    let flip = (0..n).last_true(k => light(k) == dark)!
    for i in 0..flip: word[i] = '1'
    word[flip] = 'x'
print(word)
