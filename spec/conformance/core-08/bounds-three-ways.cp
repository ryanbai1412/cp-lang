// expect: ok, output "7 3 6 6"
// rule: 08.2.1 12.11.2
fn big<T: Ord>(a: T, b: T) -> T = max(a, b)
fn small<T>(a: T, b: T) -> T where T: Ord = min(a, b)
fn tot(s: Seq<Int>) -> Int = s.sum()
print(big(3, 7), small(3, 7), tot([1, 2, 3]), tot((1..=3).map(x => x)))
