// expect: ok, output "1 1 1 1 1 1 1\n1 1 0"
// rule: 06.4.14
// rule: std.derive.hash_fields
// `Hash` is `@structural` in std: a tuple, struct, optional, position, array or
// record has it when each part does, and a scalar through its own std impl, as
// `I128`'s.
struct P(a: Int, b: Str)
domain V = 0..5
fn ok<T: Hash>(a: T) -> Int = 1
fn same<T: Hash>(a: T, b: T) -> Int = if a.hash() == b.hash(): 1 else: 0
print(ok((1, "a")), ok(P(2, "b")), ok(Some(3)), ok(V(2)), ok([1, 2]), ok((x: 1, y: 'c')), ok(I128(7)))
print(same(4, 4), same(I128(-5), I128(-5)), same(I128(-5), I128(5)))
