// DMOPC 2020 C4 P4: replay the greedy stack. `size` is how many holes there are; a throw
// asking for more than that reopens the most recent spare throws, each costing its distance.
let n: 1..=2e6 = read()
var size = 0
var cost = 0
var spare: List<Int> = []
print([0..n] i =>
    let x: 0..=i = read()
    while size < x:
        cost += i - spare.pop()!
        size += 1
    if x == size:
        size += 1
    else:
        cost += size - x - 1
        spare.push(i)
    cost)
