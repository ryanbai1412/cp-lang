// A walk over one sorted set in a list of them lets a sibling set grow
// (acceptance cf2289E).
var pool = [SortedSet.of([x]) for x in 0..2]
pool[1].add(5)
var a = 0
var b = 1
if pool[a].len < pool[b].len: (a, b) = (b, a)
for x in pool[b]: pool[a].add(x)
print(pool[a].len, pool[b].len)
