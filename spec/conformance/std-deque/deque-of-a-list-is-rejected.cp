// expect: E0303 at 3
// `Deque(xs)` is an error rather than Python's `deque(xs)` or C++'s `deque(n)`; the contents go through `Deque.of(xs)`.
let a: List<Int> = [3, 1, 3]
let d = Deque(a)
