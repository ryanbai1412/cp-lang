struct Seg:
    lo: [0..3]Int
    hi: List<Int>
var cur = Seg([5, 6, 7], [1, 2, 3])
let left = Seg([9, 1, 8], [4, 0, 9])
cur.lo min= left.lo
print(cur.lo)
cur.hi max= left.hi
print(cur.hi)
cur.lo[..] min= left.lo[..]
var a = [1, 5, 3, 7]
let b = [4, 2, 6, 0]
a max= b
print(a)
a[1..3] min= b[1..3]
print(a)
print(a[..] max= b[..], a)
let v: Int = read()
print(v & (v - 1), (v | 1) + 2, v ^ (v + 1), (v & 1) == 0)
var cnt = Map<Int, Int>()
var seen = HashMap<Str, List<Int>>()
for x in [3, 1, 3, 3, 1]:
    cnt.update(x, 0, c => c + 1)
seen.update("a", [], xs => xs + [1])
seen.update("a", [], xs => xs + [2])
print(cnt[3], cnt[1], seen["a"])
var part = [9, 9, 9, 9, 9]
part[1..4] min= [3, 12, 5]
part[3..] max= b[..2]
print(part)
let opt: [0..3]?Int = [none, some(4), some(2)]
if a[..2] min= [0, 9]: print("lower", a)
print(opt[1]! & (opt[2]! + 1) | 8, 7 ^ 1 << 2, (v & 3) < 2)
