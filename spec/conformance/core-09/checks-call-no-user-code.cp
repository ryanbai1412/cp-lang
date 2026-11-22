// expect: ok, output "call\n20"
// rule: 09.2.3
fn at() -> Int:
    print("call")
    1
let a = [10, 20, 30]
print(a[at()])
