// expect: E0702
// rule: 08.1.4
// Two fns of one name in one file whose `self`s are equally specific are no overload: neither is picked over the other.
fn kind(self: Range<Int>) -> Str = "a"
fn kind(self: Range<Int>, n: Int) -> Str = "b" // <- here
print((0..3).kind())
