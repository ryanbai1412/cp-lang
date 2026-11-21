// Part 1. Name a triple's positions i < j < k; its distances are j−i, k−j and the
// largest, k−i. Whichever position holds height k−i, two heights pin the third
// position, so every triple is reached from O(1) candidates per position, except
// "middle holds k−i, crossed" (h[i] = k−j, h[k] = j−i): there i and j share
// h[x] − x, and a √n split between small groups (all pairs) and large groups
// (solve for i, j from each k) bounds it. One predicate checks, `distinct` dedupes.
@grader
fn count_triples(h: [I]1..I.count) -> Int:
    fn ok(x: Int, y: Int, z: Int) -> ?(I, I, I) =
        let (i, j, k) = sort3(x, y, z)
        if i < j < k and (i, j, k) all in I and sorted([h[i], h[j], h[k]]) == [j - i, k - j, k - i]: (i, j, k)

    let found = Bag<(I, I, I)>()
    for x in I:
        for s in [-1, 1]:                              // x is an end holding k−i
            let y = x + s * h[x]
            if y in I:
                for m in [x + s * h[y], y - s * h[y]]: found.add(ok(x, m, y))
        let m = x + h[x]                               // middle holds k−i, h[i] = j−i
        if m in I: found.add(ok(x, m, x + h[m]))

    for group in I.group_by(x => h[x] - x):            // middle holds k−i, crossed
        if group.len <= isqrt(I.count):
            for (i, j) in group.pairs(): found.add(ok(i, j, i + h[j]))
        else:
            let g = h[group.first] - group.first
            for k in I where (k - g - h[k]) % 2 == 0:
                found.add(ok((k - g - h[k]) / 2, (k - g + h[k]) / 2, k))
    return found.distinct().len
