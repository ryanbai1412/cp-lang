// expect: ok, output "130 2 69 129 YES"
use std.bitset.{Bitset}
domain W = 0..130
var a = Bitset<W>.full()
var x = Bitset<W>.new()
x.add(W(5))
x.add(W(127))
x.toggle(W(5))
x.toggle(W(70))
var y = Bitset<W>.new()
y.set_range(W(60)..W(129))
print(a.len, x.len, y.count_range(W(2)..W(129)), a.complement().len + 129, yes(x.add(W(1)) and not x.add(W(1)) and x.remove(W(1)) and not x.remove(W(1))))
