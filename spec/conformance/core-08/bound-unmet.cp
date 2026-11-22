// expect: E0606 at 3
// rule: 08.2.1
struct F: f: Int -> Int
fn big<T: Ord>(a: T, b: T) -> T = max(a, b)
let x = big(F(x => x), F(x => x))
