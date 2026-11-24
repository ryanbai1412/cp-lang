// expect: R0501 (checked)
// rule: 10.2.7
let xs = read_all<Int>()
let y: Int = read() // <- here
print(xs.len, y)
