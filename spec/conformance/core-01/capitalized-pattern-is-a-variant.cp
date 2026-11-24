// expect: E0302
// rule: 01.3.3
let x = 3
match x:
    Foo => print(1) // <- here
    _ => print(2)
