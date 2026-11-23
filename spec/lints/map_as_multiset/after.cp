let q: 1..=200000 = read()
var bag = SortedBag<Int>()
for _ in 0..q:
    let (t, x): (1..=2, 1..=1000000000) = read()
    if t == 1:
        bag.add(x)
    else:
        bag.remove(x)
print(bag.len)
