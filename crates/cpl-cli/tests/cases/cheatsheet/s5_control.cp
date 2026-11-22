let x = 5
var word = ""
if x < 0: word = "neg"
elif x == 0: word = "zero"
else: word = "pos"
let sign = if x >= 0: 1 else: -1
let (a, b, q) = (7, 3, 2)
var (r0, r1) = (a, b)
(r0, r1) = (r1, r0 - q * r1)
var ev: List<Int> = []
for v in 0..5 if v % 2 == 0: ev.push(v)
var st: List<Int> = []
for i in (0..8).step(2).rev: st.push(i)
var q2: List<Int> = [0, 1, 2]
var popped = 0
while let y = q2.pop() and y > 0: popped += 1
var hits = 0
'outer: for r in 0..3:
    for c in 0..3:
        if r + c == 1: break 'outer
        hits += 1
print(word, sign, r0, r1 - 2, ev[2], ev[1], ev[0], st[0], st[1], st[2], st[3], popped, hits)
