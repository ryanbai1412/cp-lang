// expect: ok, output "6"
// rule: 05.10.4 06.2.5
fn twice(f: (Int) -> Int, x: Int) -> Int = f(f(x))
let k = 2
print(twice(y => y + k, 2))
