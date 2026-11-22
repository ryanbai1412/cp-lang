// expect: ok, output "5 7\n9"
// input: "5 7 0 9\n"
let xs = read_all<Int>(until: x => x == 0)
print(xs)
print(read<Int>())
