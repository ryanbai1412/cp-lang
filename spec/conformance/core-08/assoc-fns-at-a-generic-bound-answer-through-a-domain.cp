// expect: ok, output "4 3 3"
// rule: 08.1.9
// A static fn a trait declares answers on a type param bound by that trait (`P.at`, `P.count()`), and a `D: Domain` param answers `D.count` and `D.nth(k)` inside a generic impl (std/segtree.cp Positions).
trait Positions:
    fn count() -> Int
    fn at(k: Int) -> Self
impl<D: Domain> Positions for D:
    fn count() -> Int = D.count
    fn at(k: Int) -> D = D.nth(k)
fn last<P: Positions>() -> P = P.at(P.count() - 1)
fn all<P: Positions>() -> List<P> = [P.at(k) for k in 0..P.count()]
domain E = 2..5
print(last<E>(), all<E>().len, E.count)
