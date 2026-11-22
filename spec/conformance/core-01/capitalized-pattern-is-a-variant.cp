// expect: E0302 at 3
// rule: 01.3.3
let x = 3
match x:
    Foo => print(1)
    _ => print(2)
