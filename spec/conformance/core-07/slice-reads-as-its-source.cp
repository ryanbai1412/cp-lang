// expect: ok, output "2 3\n9\n3"
// rule: 07.4.17
fn first_of<K>(a: [K]Int) -> Int = a.first!
let xs = [1, 2, 3, 4, 9]
var rest = xs[..]
rest = rest.drop(1)
print(rest.first!, rest.nth(1))
var best: ?Slice<List<Int>> = None
best = xs[4..]
print(best!.first!)
print(first_of(xs[2..4]))
