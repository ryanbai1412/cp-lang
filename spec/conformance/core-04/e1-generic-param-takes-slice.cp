// expect: ok, output "2\n4"
// rule: 04a.6.1 04b.3.1 04b.4.1
fn first_of<K>(a: [K]Int) -> Int = a.first!
fn top<K>(a: [K]Int) -> K = a.argmax()!
let s = [1, 2, 3, 4, 9]
print(first_of(s[1..3]))
print(top(s[2..]))
