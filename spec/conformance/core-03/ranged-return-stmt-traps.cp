// expect: R0201
// rule: 03.1.4
// input: "2\n"
fn f(x: Int) -> 0..10:
    if x > 0:
        return x + 100 // <- here
    3
print(f(read<Int>()))
