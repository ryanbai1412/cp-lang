// AAC 4 P6. The corpus has no statement, so this dream follows the accepted C++'s idea and is
// not a rethink: a deque of pending "zero" marks seeded from the last min(n, k) values; each
// position loses one per live mark, keeps at least 1 unless a mark ends exactly here, then
// pushes its own mark and retires a[i] old ones. If k > n and some value hit 0, rotate by k % n.
let (n, k) = read (Int, Int)
domain I = [n]
var a = read [I]Int
if k == 0: print(a); return
let m = min(n, k)
var marks = Deque<Int>()
var sum = 0
for (j, x) in a.rev().take(m).enumerate():
    sum += x
    if sum > 0: sum -= 1 else: marks.push_back(j)
for (i, x) in a.items:
    marks.pop_back_while(j => j >= m - Int(i))
    let ends_here = marks.back == some(m - Int(i) - 1)
    a[i] = max(x - marks.len, if ends_here: 0 else: 1)
    marks.push_front(-Int(i) - 1)
    repeat min(x, marks.len): marks.pop_front()
if k > n and a.min()! == 0: a = a.rotated_right(k % n)
print(a)
