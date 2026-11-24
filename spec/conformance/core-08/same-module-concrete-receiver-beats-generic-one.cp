// expect: ok, output "generic concrete generic"
// rule: 08.1.4
// Two fns of one name in one file that take a `self` are overloads: `self: Range<F64>` is more specific than `self: Range<D>`, so it takes the real range and the integer one still calls the generic fn.
fn kind<D>(self: Range<D>) -> Str = "generic"
fn kind(self: Range<F64>) -> Str = "concrete"
domain D = 0..10
print((0..3).kind(), (0.0..1.0).kind(), (D(0)..D(4)).kind())
