// expect: ok, output "3\n9\n5\n11"
// A list literal where a `[D]T` is wanted and nothing else fixes `D`: `D` is the list's
// positions `0..n` (R04.4.30). When another argument fixes `D`, the list takes that one.
fn n<D: Domain>(a: [D]Int) -> Int = D.count
fn total<D>(a: [D]Int) -> Int:
    var s = 0
    for x in a: s += x
    s
fn both<D: Domain>(a: [D]Int, b: [D]Int) -> Int = D.count + total(a) + total(b) - 3
print(n([1, 2, 3]))
print(total([4, 5]))
domain V = 0..2
let w = [V] v => Int(v) + 1
print(both([1, 2], w))
print(both(w, [4, 5]))
