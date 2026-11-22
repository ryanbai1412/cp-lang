// expect: E0802 at 3
// rule: 09.1.1
fn f():
    print(1)
    with checks(overflow: off)
    print(2)
