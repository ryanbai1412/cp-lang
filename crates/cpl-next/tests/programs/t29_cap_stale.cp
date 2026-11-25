// expect: ok, output "6"
// input: "5\n"
fn go(n: Int) -> Int:
    var k = 1
    let f = (x: Int) => x + k
    k = n
    f(1)
print(go(5))
