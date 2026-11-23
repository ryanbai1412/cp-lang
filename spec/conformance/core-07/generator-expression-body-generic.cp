// expect: ok, output "15"
// rule: 07.2.10
// A generic generator with an expression body returns that sequence.
fn firsts<T>(xs: List<T>) -> yields T = xs.take(2)
print(firsts([7, 8, 9]).sum())
