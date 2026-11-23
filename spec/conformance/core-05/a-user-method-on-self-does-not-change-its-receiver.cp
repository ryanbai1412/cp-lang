// expect: ok, output "0 x"
// rule: 06.1.2
domain City = 1..=3
struct Pair<A, B>(first: A, second: B):
    fn swap(self) -> Pair<B, A> = Pair(second, first)
let p = Pair(City.first!, 'x')
let q = p.swap()
print(p.first.ord, q.first)
