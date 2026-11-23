// expect: ok
// rule: 09.2.2 09.3.2
// tool: build -o -
// has: "put_int"
// lacks: "cpl::at(xs"
let j = read<Int>()
let xs = [1, 2, 3]
print(xs[j])
