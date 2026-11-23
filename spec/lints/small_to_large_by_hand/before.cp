let (n, q): (1..=100000, 1..=100000) = read()
var sets: List<SortedSet<Int>> = [SortedSet.of([x]) for x in 0..n]
var id: List<Int> = [x for x in 0..n]
for _ in 0..q:
    let (x, y): (0..n, 0..n) = read()
    if x == y:
        continue
    var big = id[x]
    var small = id[y]
    if sets[big].len < sets[small].len:
        (big, small) = (small, big)
    for w in sets[small]:
        sets[big].add(w)
    sets[small] = SortedSet.of([])
    id[x] = big
    id[y] = small
    print(sets[big].len)
