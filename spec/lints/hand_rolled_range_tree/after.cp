domain Pos = 1..=8
var bit = Fenwick<Pos, Int>()
bit.add(Pos(3), 1)
print(bit.prefix(Pos(4).r))
