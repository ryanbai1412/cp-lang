let (n, q): (1..=100000, 1..=100000) = read()
var sets: List<SortedSet<Int>> = [SortedSet.of([x]) for x in 0..n]
for _ in 0..q:
    let (x, y): (0..n, 0..n) = read()
    if x == y:
        continue
    sets[x].absorb(mut sets[y])
    print(sets[x].len)
