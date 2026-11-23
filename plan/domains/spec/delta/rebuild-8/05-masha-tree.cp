// 1741D Masha and a Beautiful Tree
// in: t; m (a power of 2); p_1..p_m (leaf values, a permutation). out: per test, fewest subtree swaps to sort the leaves, or -1.
@!multitest
let m = read<1..=262144>()
domain Leaf = 0..m
let p = read<[Leaf]1..=262144>()

fn swaps(r: Range<Leaf>) -> ?Int:
    let (left, right) = r.halve() else: return 0
    if p[r].max()! - p[r].min()! + 1 != r.len: return None
    let fix = if p[left].min()! > p[right].min()!: 1 else: 0
    swaps(left) + swaps(right) + fix

print(swaps(Leaf.range) ?? -1)
