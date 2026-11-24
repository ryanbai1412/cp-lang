// expect: E0501
// rule: 08.5.1
struct A: x: Int
struct B(A): y: Int // <- here
