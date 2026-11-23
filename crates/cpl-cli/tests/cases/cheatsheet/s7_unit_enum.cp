enum Dir = N | E | S | W
var cnt: [Dir]Int = 0
cnt[Dir.E] += 1
print(Dir.count, cnt[Dir.E])
