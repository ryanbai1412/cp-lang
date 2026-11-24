use std.marks.{Interner}
struct Names:
    ids: Interner<Str>
    count: Int
var t = Interner<Str>()
for w in ["pear", "apple", "zebra", "apple", "kiwi", "fig", "pear"]: t.id(w)
print(t)
print("[{t}]")
print(t.len, t)
var nums = Interner<Int>()
for x in [977, 3, 500, 3, 41, 1000000007, 2]: nums.id(x)
print(nums)
let n = Names(t.clone(), t.len)
print(n)
print([t.clone(), t.clone()])
t.clear()
print("[{t}]")
t.id("b")
t.id("a")
print(t)
