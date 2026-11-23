// expect: ok, output "20 10\n1 0\n30 20 10\n2 1 0"
// rule: 07.3.5
// `a.rev.take_last(k)` cuts in the order `a.rev` walks: the source's first two, still walked backwards and keeping the source's keys; a `k` past the length takes it all.
let a = [10, 20, 30]
print(List(a.rev.take_last(2)))
print([k for (k, x) in a.rev.take_last(2).items])
print(List(a.rev.take_last(5)))
print([k for (k, x) in a.rev.take_last(5).items])
