// expect: ok, output "4 6\n3 0"
// rule: R04.3.4 R04.8.10
// input: "4\n"
// A table over an anonymous range, where an array over another anonymous range is wanted,
// gives its cells to that array as a list does: such an array takes any `Int`.
let n: Int = read()
let a: [0..n]Int = [0..n] i => i * 2
print(a.len, a[3])
let b: [0..3]Int = [0..n - 1] i => i + 1
let e: [0..0]Int = [0..n - n] i => i
print(b[2], e.len)
