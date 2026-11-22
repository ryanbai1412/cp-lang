// expect: ok, output "5 6"
// rule: 09.5.4
fn add(a: Int, b: Int) -> Int = @cpp("return a + b;")
var y = 0
@trusted:
    y = 6
print(add(2, 3), y)
