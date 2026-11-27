// expect: ok, output "0 3 1 0 3\n1 2 3 YES\n3 YES"
// rule: R10.2.2 R10.2.4 R09.1.1
// input: "1 3\n3 2 1 4\n1 2\n"
// `read(base: b)` and `read(from: s, base: b)` take `b` over the surrounding `io.base`, at every part
// of the read; a range with explicit ends never shifts.
domain E = [4]
let a: E = read(base: 1)
let b: 1..=4 = read(base: 1)
let xs: List<E> = read(base: 1)
print(Int(a), b, Int(xs[0]), Int(xs[1]), Int(xs[2]))
with io(base: 1):
    let p: (E, E) = read(base: 0)
    print(Int(p.0), Int(p.1), Int(read<E>(from: "4")!), yes(read<E>(from: "4", base: 0) == None))
print(Int(read<E>(from: "4", base: 1)!), yes(read<E>(from: "4") == None))
