// expect: R0201
// rule: 04.4.24
// input: "99\n"
// The comprehension is the explicit conversion: it checks each value it stores, so the second one traps.
let n: Int = read()
let zs: List<Int> = [n, n + 1, n + 2]
var ys: List<0..100> = [x for x in zs] // <- here
print(ys)
