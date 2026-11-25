// expect: E0608
// rule: R05.2.2, R05.5.3
// A generic fn that stores its `T` argument keeps it, so an owned list passed to it moves.
fn put<T>(mut into: List<T>, x: T):
    into.push(x)
var xs = [1, 2]
var keep: List<List<Int>> = []
for i in 0..2:
    put(mut keep, xs) // <- here
print(keep.len)
