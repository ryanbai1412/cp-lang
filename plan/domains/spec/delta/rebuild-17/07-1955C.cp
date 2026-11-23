// 1955C Inhabitant of the Deep Sea: the front takes ceil(k/2) hits, the back floor(k/2).
@!multitest
let n: 1..=2e5 = read()
let k: 1..=1e15 = read()
let a = read<[n]1..=1e9>()
if a.sum() <= k:
    print(n)
    return
var front = (k + 1) / 2
var back = k / 2
var sunk = 0
for x in a:
    if x > front: break
    front -= x
    sunk += 1
for x in a.rev:
    if x > back: break
    back -= x
    sunk += 1
print(sunk)
