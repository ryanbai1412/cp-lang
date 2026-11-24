// expect: ok, output "a b"
// rule: 08.1.4
// Two fns of one name in one file overlap only if one call could take both: the same `self` with another number of parameters is an overload.
fn kind(self: Range<Int>) -> Str = "a"
fn kind(self: Range<Int>, n: Int) -> Str = "b"
print((0..3).kind(), (0..3).kind(1))
