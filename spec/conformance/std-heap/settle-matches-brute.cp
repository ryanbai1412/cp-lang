// expect: ok, output "2 none\nNO a 2 YES\nb 3 none 2\na 5\nYES"
use std.heap.{Settle}
use std.rand.{Rng}
var s: Settle<Char, Int> = Settle()
s.offer('a', 5)
s.offer('a', 2)
s.offer('a', 4)
print(s.get('a'), s.get('b'))
s.offer('b', 3)
print(s.settled('a'), s.pop(), s.settled('a'))
s.offer('a', 1)
print(s.pop(), s.pop(), s.get('a'))
var t: Settle<Char, Int> = Settle(best: max)
t.offer('a', 5)
t.offer('b', 3)
print(t.pop())
var rng = Rng(3)
var u: Settle<Int, Int> = Settle()
var bestv = List.filled(30, 1000)
var fin = List.filled(30, false)
var ok = true
for _ in 0..500:
    if rng.int(0, 2) > 0:
        let k = rng.int(0, 29)
        let p = rng.int(0, 99)
        u.offer(k, p)
        if not fin[k]: bestv[k] = min(bestv[k], p)
    else:
        var bk = -1
        for k in 0..30:
            if not fin[k] and bestv[k] < 1000 and (bk < 0 or bestv[k] < bestv[bk]): bk = k
        let got = u.pop()
        if bk < 0:
            if got != None: ok = false
        elif let (k, p) = got:
            if p != bestv[bk] or bestv[k] != p or fin[k]: ok = false
            fin[k] = true
        else:
            ok = false
print(ok)
