// TLE '17 Contest 5 P6: a circuit of m NAND gates over n inputs, the last
// gate being the output. Set every input to 0, 1 or x so that the output
// changes with x, or print all zeros if no input can ever change it.
// original: corpus/submissions/5142110.cpp
let n: 1..=1e5 = read()
let m: 1..=2e5 = read()
domain Input = 1..=n
domain Gate = 1..=m
domain Wire = Input | Gate                                          // WISH K11: a wire is an input or a gate
let gates: [Gate](a: Wire, b: Wire) = read()

// the output when the first `ones` inputs are on and the rest are off
fn light(ones: Int) -> Bool:
    @memo                                                           // WISH: a memo that lives for one call of `light`
    fn out(w: Wire) -> Bool = match w:
        Input(i) => i.label <= ones
        Gate(g) => not (out(gates[g].a) and out(gates[g].b))
    out(Gate.last)

// If all off and all on differ, somewhere along the way one more input
// turned on flips the light: that input is x, with ones before and zeros after.
let dark = light(0)
if dark == light(n):
    print("0" * n)
else:
    let flip = (0..n).last_where(k => light(k) == dark)!            // WISH R1
    print("1" * flip + "x" + "0" * (n - flip - 1))
