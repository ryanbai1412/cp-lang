// expect: ok, output "1 none 3 none"
// rule: std.seq.next_smaller
// `by: desc` works on a sequence of domain values: a generic fn value takes its type from the parameter it fills.
domain V = 0..10
domain I = 0..4
let a: [I]V = [I] i => V([2, 5, 1, 4][i.ord])
let nx = a.next_smaller(by: desc)
print(nx[I(0)], nx[I(1)], nx[I(2)], nx[I(3)])
