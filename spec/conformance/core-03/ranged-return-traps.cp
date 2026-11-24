// expect: R0201
// rule: 03.1.4
// input: "2147483648\n"
fn f(x: Int) -> 0..10 = x // <- here
print(f(read<Int>()))
