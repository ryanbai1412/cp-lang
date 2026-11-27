// expect: ok, output "3 0 7 0\n2 0 YES\n5 YES"
// rule: R10.2.10 R10.2.4
// input: "3 -1 7\n-1\n2 -1\n"
// engines: next
// Under `none: k` the token `k` is a `?T`'s `None`, at every part of the read and from a string;
// any other token, `-10` too, reads as the `T`.
let a: (?Int, ?Int, ?Int, ?Int) = read(none: -1)
print(a.0 ?? 0, a.1 ?? 0, a.2 ?? 0, a.3 ?? 0)
let xs: [2]?Int = read(none: -1)
print(xs[0] ?? 0, xs[1] ?? 0, yes(read<?Int>(from: "-10", none: -1)! == Some(-10)))
let five = read<?Int>(from: "5", none: -1)!
print(five ?? 0, yes(read<?Int>(from: "-1", none: -1)! == None))
