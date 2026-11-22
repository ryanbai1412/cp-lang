// expect: ok, output "1\n1 2\n-1 7"
// input: "3\n2\n1 2\n-1 7"
// rule: 10.2.2
let m = read<1..=10>()
domain E = [m]
let x = read<E>(base: 1)
print(Int(x))
with io(base: 1):
    let a = read<[2]E>(base: 0)
    print(Int(a[0]), Int(a[1]))
let b = read<[2]?Int>(none: -1)
print(b, none: -1)
