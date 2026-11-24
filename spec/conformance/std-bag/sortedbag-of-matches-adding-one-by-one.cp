// expect: ok, output "ok"
// `SortedBag.of` builds a bag of at most one leaf by sorting and counting runs; it must hold
// what adding the elements one by one holds, for every size around the leaf width (32),
// with duplicates, sorted and reversed inputs.
var bad = 0
for n in 0..=40:
    for shape in 0..3:
        let xs = [if shape == 0: (i * 7) % 5 else: if shape == 1: i / 3 else: n - i for i in 0..n]
        let b = SortedBag.of(xs)
        var c: SortedBag<Int> = SortedBag()
        for x in xs: c.add(x)
        if b.to_list() != c.to_list() or b.len != c.len or b.distinct != c.distinct: bad += 1
        var d = SortedBag.of(xs)
        d.add(2)
        c.add(2)
        if d.pop_first() != c.pop_first() or d.take_first(x => x >= 3) != c.take_first(x => x >= 3) or d.to_list() != c.to_list(): bad += 1
print(if bad == 0: "ok" else: "bad {bad}")
