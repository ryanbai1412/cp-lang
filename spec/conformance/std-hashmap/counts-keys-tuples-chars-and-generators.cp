// expect: ok, output "2 0 2\n2 0 3\n3 2 0\n0"
let c = counts([(1, 2), (1, 2), (3, 4)])
print(c[(1, 2)], c[(5, 5)], c.distinct)
let d = counts("abca")
print(d['a'], d['z'], d.distinct)
let e = counts(x % 3 for x in 0..7)
print(e[0], e[2], e[5])
let none: List<Int> = []
print(counts(none).distinct)
