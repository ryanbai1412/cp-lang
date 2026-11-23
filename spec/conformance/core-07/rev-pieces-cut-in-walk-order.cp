// expect: ok, output "20 10\n30 20\n10\n30 20\n1 0\n20"
// rule: 07.3.5
// A piece of a reversed view is cut in the order the view walks: `a.rev.drop(1)` drops the 30 that `a.rev` walks first. Its keys stay the source's, and `.first` is its first element in walk order (P0: every piece came out mirrored, a silent wrong answer).
let a = [10, 20, 30]
print(List(a.rev.drop(1)))
print(List(a.rev.take(2)))
print(List(a.rev.take_last(1)))
print(List(a.rev.drop_last(1)))
print([k for (k, x) in a.rev.drop(1).items])
print(a.rev.drop(1).first!)
