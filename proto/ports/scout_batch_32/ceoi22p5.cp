// CEOI '22 P5 - Measures (DMOJ: ceoi22p5)
// Ideal v2 port. original: corpus/submissions/4733784.cpp
//
// People stand on a line and must end up at least d apart; everyone moves at
// unit speed. n people are there from the start and m more arrive one by one;
// after each arrival print the least time needed (a multiple of 1/2).
//
// For people i < j in sorted order the answer is at least
// ((j - i) * d - (x_j - x_i)) / 2, and the maximum over pairs is enough. All
// arrivals are known up front, so each person gets a fixed leaf in sorted
// order and a segment tree merges (count, best left end, best right end, best pair).
let (n, m, d): (1..=2e5, 1..=2e5, 1..=1e9) = read()
let xs: [0..n + m](0..=1e9) = read()

// WISH X4: equal positions need distinct leaves, and `compress` merges equal
// values. Compressing (x, i) pairs gives a stable rank, at the price of
// looking it up with the same pair again.
let slot = compress((x, i) for (i, x) in xs.items)

struct Span:
    cnt: Int = 0
    left: ?Int = none       // max of (people before it in the span) * 2d - 2x
    right: ?Int = none      // max of 2x + (people after it in the span) * 2d
    best: Int = 0           // twice the answer inside the span
    fn of(x: Int) -> Span = Span(1, -2 * x, 2 * x, 0)
    fn then(self, o: Span) -> Span:
        // WISH X1 again: shifting an optional by `cnt * 2 * d` needs a helper.
        let cross = if let (a, b) = (self.right, o.left): (a + b) / 2 + d else: 0
        Span(
            self.cnt + o.cnt,
            max(self.left, shift(o.left, self.cnt * 2 * d)),
            max(o.right, shift(self.right, o.cnt * 2 * d)),
            max(self.best, o.best, cross),
        )
fn shift(v: ?Int, by: Int) -> ?Int = if let x = v: x + by else: none

var st = SegTree<slot.D, Span>(Span(), (a, b) => a.then(b))
for (i, x) in xs.items:
    st.set(slot.at((x, i)), Span.of(x))
    if i >= n:
        let t = st.fold(..).best
        print(if t % 2 == 1: "{t / 2}.5" else: "{t / 2}")
