// expect: ok, output "20 5"
// (spec) R12: body runs inside the generator, so it sees k change. Today prints 6 5. Spec does not say whether a plain top-level var read by a running generator may be written by the body: QUESTION
// Generator reads a global scalar by name; the body writes it. Model 4: reject (or define).
var k = 1
fn gen(n: Int) -> yields Int:
    for i in 0..n: yield i * k
var s = 0
for v in gen(4):
    k += 1
    s += v
print(s, k)
