// expect: R0201
// rule: R03.1.4
// input: "10\n"
// R03.1.4: a fn's range return type checks what it returns.
fn f(x: Int) -> 0..10:
    x  // <- here
let n: Int = read()
print(f(n - 1))
print(f(n))
