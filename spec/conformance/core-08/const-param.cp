// expect: ok, output "5 3"
// rule: 08.2.4
struct Buf<const N: Int>: x: Int
fn cap<const N: Int>(b: Buf<N>) -> Int = N
let m: Mod<7> = 10
print(cap(Buf<5>(0)), m)
