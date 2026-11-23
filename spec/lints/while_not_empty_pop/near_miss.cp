var q = [3, 1, 2]
var other = [5]
var total = 0
// The test and the pop name different lists: nothing is said twice.
while not other.is_empty:
    let x = q.pop()!
    total += x
    other.pop()
// The pop is not the first thing the loop does.
while not q.is_empty:
    total += 1
    let y = q.pop()!
    total += y
print(total)
