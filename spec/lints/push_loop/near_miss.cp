let raw = [3, 1, 2]
var seen: List<Int> = []
for x in raw:
    if not seen.contains(x): seen.push(x)
    print(seen.len)
