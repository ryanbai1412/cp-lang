// expect: E0603 at 3
// rule: 05.5.2
// `Box(xs)` holds the lent parameter, so the result is lent; returning it needs `lends Box` or `Box(xs.clone())`.
struct Box(v: List<Int>):
    fn of(xs: List<Int>) -> Box = Box(xs)
print(Box.of([1]).v)
