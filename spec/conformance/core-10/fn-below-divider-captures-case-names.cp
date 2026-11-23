// expect: ok, output "4\n6"
// rule: 10.1.1 06.2.1
// input: "2\n3\n5\n"
// A `fn` below the divider captures that case's names.
@!multitest
let k = read<Int>()
fn add(x: Int) -> Int = x + k
print(add(1))
