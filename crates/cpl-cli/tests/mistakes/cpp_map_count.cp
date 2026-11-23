// here: 5
// C++'s `m.count(k)` (self-run mistakes): a key is tested with `in`.
var m = Map<Int, Int>()
m[3] = 1
if m.count(3):
    print(1)
