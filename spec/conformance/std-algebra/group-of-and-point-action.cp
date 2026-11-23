// expect: ok, output "-5 5\n13"
use std.algebra.{point_action}
print(Group<Int>.of(+).inv(5), Group<Int>.of(^).inv(5))
print(point_action(id: 0, compose: (a, b) => a + b, apply: (f, x) => x + f, over: max).apply(3, 10, 4))
