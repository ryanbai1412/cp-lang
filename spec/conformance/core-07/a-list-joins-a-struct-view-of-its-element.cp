// expect: ok, output "3 50"
// rule: 04a.2.3
// `xs + view` joins any struct view whose Seq impl walks `xs`'s element type, as it joins any other sequence.
struct Twice<A>(a: A)
impl Seq<(Int, Int)> for Twice<A: Seq<Int>>:
    fn iter(self) -> yields (Int, Int):
        for x in a:
            yield (x, x * 10)
fn twice<A: Seq<Int>>(a: A) -> Twice<A> = Twice(a)
let joined = [(0, 0)] + twice([4, 5])
print(joined.len, joined[2].1)
