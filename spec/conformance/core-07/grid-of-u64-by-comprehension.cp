// expect: ok, output "1 11"
// input: "3\n"
// rule: 07.4.12
// A grid of U64 built by a comprehension compiles: C++ default-constructs and casts its cells by a one-word type name.
let n: 1..=10 = read()
domain R = 0..n
domain C = 0..2
let g: [R, C]U64 = [R, C] (r, c) => U64(r.ord * 5 + c.ord)
print(Int(g[R(0), C(1)]), Int(g[R(2), C(1)]))
