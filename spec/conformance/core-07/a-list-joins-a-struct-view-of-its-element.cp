// expect: ok, output "3 50"
// rule: 04a.2.3
// `xs + view` joins any struct view whose Seq impl walks `xs`'s element type, as it joins any other sequence.
struct Twice<A, T>(a: A)
impl Seq<(T, T)> for Twice<A: Seq<T>, T: Mul>:
    fn iter(self) -> yields (T, T):
        for x in a:
            yield (x, x * 10)
fn twice<A: Seq<T>, T: Mul>(a: A) -> Twice<A, T> = Twice(a)
let joined = [(0, 0)] + twice([4, 5])
print(joined.len, joined[2].1)
