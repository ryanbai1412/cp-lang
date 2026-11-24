// expect: ok, output "1 2 10 a\n3 4 20 b"
// rule: 04b.5.3
// A struct view passed for a bound's sequence is the sequence its own Seq impl walks, so a view nested inside another view's Positional side pairs by its walk element, not by a type argument.
struct Both<A, B, T, U>(a: A, b: B)
impl Seq<(T, U)> for Both<A: Seq<T>, B: Positional<U>, T, U>:
    fn iter(self) -> yields (T, U):
        var i = 0
        for x in a:
            if i == b.len: return
            yield (x, b.nth(i))
            i += 1
impl Sized for Both<A: Sized, B: Sized, T, U>:
    @get fn len(self) -> Int = min(a.len, b.len)
impl Positional<(T, U)> for Both<A: Positional<T>, B: Positional<U>, T, U>:
    fn nth(self, k: Int) -> (T, U) = (a.nth(k), b.nth(k))
fn both<A: Seq<T>, B: Positional<U>, T, U>(a: A, b: B) -> Both<A, B, T, U> = Both(a, b)
for ((u, v), (l, r)) in both([(1, 2), (3, 4)], both([10, 20], ["a", "b"])):
    print(u, v, l, r)
