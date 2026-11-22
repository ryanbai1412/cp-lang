// DMOPC 2020 C4 P4: replay the greedy stack. `size` is how many elements are settled; an element
// asking for more than that reopens the most recent unsettled ones, each costing its distance.
let a = read [read Int]Int
var size = 0
var cost = 0
var unsettled: List<a.Key> = []
for (i, x) in a.items:
    while size < x:
        cost += i - unsettled.pop()!
        size += 1
    if x == size:
        size += 1
    else:
        cost += size - x - 1
        unsettled.push(i)
print(cost)
