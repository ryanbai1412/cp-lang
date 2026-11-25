// expect: R0203
// rule: contract 4: the body runs in the caller
// input: "2\n0\n"
// A trap inside a generator's body runs in the caller's frame: reported at the body's line.
fn gen(n: Int) -> yields Int:
    for i in 0..n: yield i
let n: Int = read()
for v in gen(n): print(10 / (v - 0 * read<Int>()) )  // <- here
