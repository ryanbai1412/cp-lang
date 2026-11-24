// expect: ok, output "7 -1 2 0"
// rule: 08.1.9
// Two impls of one trait choose different associated types, one of them for a fn type, and a generic fn returns whichever the argument's impl chose (decision 456).
trait Pick:
    type Found
    fn pick(self, xs: List<Int>) -> Self.Found
struct At: i: Int
impl Pick for At:
    type Found = ?Int
    fn pick(self, xs: List<Int>) -> ?Int = if self.i < xs.len: Some(xs[self.i]) else: None
impl Pick for Int -> Bool:
    type Found = Int
    fn pick(self, xs: List<Int>) -> Int:
        var n = 0
        for x in xs:
            if self(x):
                n += 1
        return n
fn both<P: Pick>(p: P, a: List<Int>, b: List<Int>) -> (P.Found, P.Found) = (p.pick(a), p.pick(b))
let (x, y) = both(At(2), [5, 6, 7], [8])
let (m, n) = both((v: Int) => v % 2 == 0, [1, 2, 4], [])
print(x!, y ?? -1, m, n)
