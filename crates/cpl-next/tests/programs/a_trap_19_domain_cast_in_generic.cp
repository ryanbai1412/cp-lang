// expect: R0102
// rule: diagnostics R0102
// input: "3 5\n"
// Typed index made from an Int outside the domain, in a generic fn (hidden size passed down).
fn at<D>(a: [D]Int, i: Int) -> Int:
    a[D(i)]  // <- here
let (n, k): (Int, Int) = read()
domain V = 0..n
var a: [V]Int = 7
print(at(a, k))
