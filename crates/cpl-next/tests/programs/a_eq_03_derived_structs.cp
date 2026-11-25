// expect: ok, output "YES NO YES NO YES\nYES YES\nYES YES"
// rule: R08.1.6
// A struct gets `Eq` and lexicographic `Ord` from its fields, unless it writes its own impl.
struct P:
    x: Int
    y: Int
struct Box<T>:
    v: T
    tag: Char
struct Odd:
    k: Int
impl Eq for Odd:
    fn eq(a, b) = a.k % 2 == b.k % 2
let a = P(1, 2)
print(yes(a == P(1, 2)), yes(a == P(1, 3)), yes(a < P(1, 3)), yes(P(2, 0) < a), yes(a >= P(0, 9)))
print(yes(Box(P(1, 2), 'a') == Box(P(1, 2), 'a')), yes(Box(3, 'a') < Box(3, 'b')))
print(yes(Odd(1) == Odd(3)), yes((a, 1) == (P(1, 2), 1)))
