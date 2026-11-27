// expect: ok, output "0 2\n1 3\n1 3 | 2 5\n[1] 3\n1 2 3\n0 1"
// rule: R10.2.2 R10.3.2
// engines: next
// A position of a counted domain prints in the base of the `io` around the print or the
// interpolation, everywhere it sits; an explicit range and a cut print as their integers.
domain E = [4]
let a = E.nth(0)
let b = E.nth(2)
print(a, b)
with io(base: 1):
    print(a, b)
    print([a, b], sep: " ", end: " | ")
    print(2, 5)
    print("[{a}] {b}")
    let r: 1..=4 = 1
    print(r, 2, 3)
print(E.cuts.nth(0), E.cuts.nth(1))
