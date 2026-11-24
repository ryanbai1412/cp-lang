// expect: E0603
// rule: 05.5.2
// `Box(xs)` holds the lent parameter, so the result is lent; returning it needs `lends Box` or `Box(xs.clone())`.
struct Box(v: List<Int>):
    fn of(xs: List<Int>) -> Box = Box(xs) // <- here
print(Box.of([1]).v)
