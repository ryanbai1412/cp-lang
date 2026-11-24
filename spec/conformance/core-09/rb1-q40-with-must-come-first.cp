// expect: E0802
// rule: 09.1.1
fn f():
    print(1)
    with checks(overflow: off) // <- here
    print(2)
