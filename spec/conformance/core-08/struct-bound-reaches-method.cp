// expect: ok, output "5"
// rule: 08.2.2
struct Acc<T: Add + Sub>(a: T, b: T):
    fn net(self) -> T = a + b - b + b
print(Acc(2, 3).net())
