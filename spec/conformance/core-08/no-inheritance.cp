// expect: E0501 at 2
// rule: 08.5.1
struct A: x: Int
struct B(A): y: Int
