// expect: ok, output "concrete generic"
// rule: 08.1.4
// Two fns of one name in one file that take a `self` are overloads: `self: Range<Int>` is more specific than `self: Range<D>`, so it takes the integer range and the domain one still calls the generic fn.
fn kind<D>(self: Range<D>) -> Str = "generic"
fn kind(self: Range<Int>) -> Str = "concrete"
domain D = 0..10
print((0..3).kind(), (D(0)..D(4)).kind())
