// expect: ok, output "1 x"
// rule: 05.4.1
domain City = 1..=3
struct Pair<A, B>(first: A, second: B):
    fn swap(self) -> Pair<B, A> = Pair(second, first)
let p = Pair(City.first!, 'x')
let q = p.swap()
print(p.first.ord, q.first)
