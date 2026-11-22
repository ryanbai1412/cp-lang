// etsr23r1p4: range add, range floor division, range min and range sum. Floor division is the
// segment-tree-beats operation: a node stops once its whole span moves by one shared amount.
let (n, q) = read<(1..=200_000, 1..=200_000)>()
domain I = 1..=n
var t = BeatsSegTree(read<[I]Int>())
for _ in 0..q:
    let (ty, l, r) = read<(1..=4, I, I)>()
    if ty == 1:
        t.add(l..=r, read<Int>())
    elif ty == 2:
        t.chdiv(l..=r, read<1..=1_000_000_000>())
    elif ty == 3:
        print(t.min_val(l..=r)!)
    else:
        print(t.sum(l..=r))
