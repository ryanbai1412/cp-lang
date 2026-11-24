// expect: E0314
// rule: 02.7.4
enum E = A | B
let e = E.A
match e: // <- here
    A => print(1)
