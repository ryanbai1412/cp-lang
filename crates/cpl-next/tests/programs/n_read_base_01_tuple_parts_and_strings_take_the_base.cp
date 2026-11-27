// expect: ok, output "3 2 3 3\n0 2 3\n3 0"
// rule: R10.2.2 R10.2.3 R10.2.4
// input: "3 2 4 3\n3 1 3 4\n"
// `read(base:)` reads a dependent tuple part by part with its base, and `parse` follows the
// caller's `io.base`; a range with explicit ends never shifts.
domain E = [4]
let (n, k): (1..=10, 1..=n) = read(base: 1)
let (a, b): (E, 1..=n) = read(base: 1)
print(n, k, Int(a), b)
let xs: List<E> = read(base: 1)
print(Int(xs[0]), Int(xs[1]), Int(xs[2]))
fn under_one(s: Str) -> Int uses io = Int(s.parse<E>()!)
with io(base: 1):
    print(under_one("4"), Int(read<E>(from: "1")!))
