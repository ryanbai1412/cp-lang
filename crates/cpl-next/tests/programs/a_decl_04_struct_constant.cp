// expect: ok, output "5"
// rule: R02.6
// A constant declared inside a struct body.
struct K:
    x: Int
    let BASE = 5
    fn base(self) -> Int = BASE
print(K(0).base())
