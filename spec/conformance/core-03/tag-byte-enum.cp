// expect: ok, output "18446744073709551615 2"
// rule: 03.2.6
enum H:
    A(x: U64)
    B(y: U64, z: U64)
fn f(h: H) -> U64 = match h: A(x) => x, B(y, z) => y ^ z
print(f(H.A(18446744073709551615)), f(H.B(1, 3)))
