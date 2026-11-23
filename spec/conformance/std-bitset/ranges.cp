// expect: ok, output "5 8 0 2 8 0 2 7 7 10"
domain D = 0..10
var a = Bitset<D>.of([D(1), D(3), D(8)])
a.set_range(D(2)..D(5))
var b = Bitset<D>.of([D(1), D(3), D(8)])
b.set_range(D(0)..D(4), on: false)
var c = Bitset<D>.of([D(1), D(3), D(8)])
c.flip_range(D(0)..D(4))
var d = Bitset<D>.of([D(1), D(3), D(8)])
d.shift_down(1)
let full = Bitset<D>.full()
print(a.len, [Int(x) for x in b.each()], [Int(x) for x in c.each()], [Int(x) for x in d.each()], Bitset<D>.of([D(1), D(3), D(8)]).complement().len, full.len)
