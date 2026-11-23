// TLE '17 Contest 5 P6, as it builds today: "the last count that still
// works" is a first on the negation with a `- 1` (R1).
// original: corpus/submissions/5142110.cpp
let n: 1..=1000 = read()
let m: 1..=1000 = read()
domain Wire = 1..=n + m
domain Gate = 1..=m

// every gate reads two earlier wires
fn random_gates() -> [Gate](a: Wire, b: Wire):
    [Gate] g => (Wire(rand(1, n + g.label - 1)), Wire(rand(1, n + g.label - 1)))

let gates: [Gate](a: Wire, b: Wire) = read(gen: random_gates)

// the output of the last gate when the first `ones` inputs are on and the rest off
var value: [Wire]?Bool = none
fn out(w: Wire) -> Bool:
    if let v = value[w]: return v
    let g = gates[Gate(w.label - n)]
    let v = not (out(g.a) and out(g.b))
    value[w] = v
    v
fn light(ones: Int) -> Bool:
    for w in Wire:
        if w.label <= n: value[w] = w.label <= ones
        else: value[w] = none
    out(Wire.last)

// If all off and all on differ, some input flips the light when it is the
// only one between a block of ones and a block of zeros.
let dark = light(0)
var word: [n]Char = '0'
if dark != light(n):
    let flip = (0..=n).first_true(k => light(k) != dark)! - 1
    for i in 0..flip: word[i] = '1'
    word[flip] = 'x'
print(word)
