// expect: E0503
// tool: check
// has: "it is `.cut`"
let p = Prefix<Int>([3, 1, 4])
print(p.prefix(2)) // <- here
