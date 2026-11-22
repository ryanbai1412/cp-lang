// cp: 0.1 9a005b0+dirty
let n: 1..=10 = read()
domain I = 1..=n
var known: [I]?Bool = none
for i in I:
    if i.idx % 3 == 1: known[i] = true
    elif i.idx % 3 == 2: known[i] = false
print(count(known[i] == none for i in I), count(known[i] == true for i in I), yes(known[I.first!] ?? true))
