// A map keyed by a pair, and by a record holding a pair, hashes its key field by field.
domain Row = 0..4
var seen: HashMap<(Int, Int), Int> = []
seen[(1, 2)] = 3
seen[(2, 1)] += 4
var deep: HashMap<(Row, (Int, Bool)), Int> = []
deep[(Row(2), (7, true))] = 9
print(seen.get((1, 2)) ?? 0, seen.get((2, 1)) ?? 0, seen.get((3, 3)) ?? 0, deep.get((Row(2), (7, true))) ?? 0, (1, 2) in seen)
