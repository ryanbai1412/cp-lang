let n: Int = read()
domain D = 0..n
var acc = 0
for i in D.all.rev:
    acc = acc * 3 + i.ord
print(acc)
for i in D.all: print(i, end: " ")
print()
let r = 2..n
for x in r:
    if x == 4: continue
    if x == 7: break
    print(x, end: " ")
print()
for x in r.rev: print(x, end: " ")
print()
let empty = n..2
for x in empty: print(x)
for x in empty.rev: print(x)
var total = 0
for x in 0..n:
    for y in (x..n).rev:
        total += x * y
print(total)
