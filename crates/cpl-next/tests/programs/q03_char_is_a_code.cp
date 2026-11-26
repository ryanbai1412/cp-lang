// expect: R0201
// input: 200
// A `Char` is a code in 0..=127 (R02.2): it orders by code, converts both ways, and
// `Char(i)` outside the range traps.
let a = 'a'
let b = Char(Int(a) + 1)
print(a, b, yes(a < b), Int(b))
var cs: List<Char> = List.new()
cs.push('x')
cs.push(b)
print(cs.get(1), cs.len)
let n = read<Int>()
print(Char(n)) // <- here
