// A local list that starts empty and never leaves its fn is one buffer in the C++,
// cleared on every call. What it holds must not leak from one call into the next.
fn spread(x: Int) -> Int:
    var xs: List<Int> = []
    for i in 0..x: xs.push(i * x)
    if xs.len > 3: return xs[3]
    var total = 0
    for v in xs: total += v
    total + xs.len

// Runs twice at once, so each call needs its own list.
fn depth(x: Int) -> Int:
    var seen: List<Int> = []
    seen.push(x)
    if x > 0: seen.push(depth(x - 1))
    seen[0] + seen.len

// The list leaves the fn, so it is built fresh.
fn upto(x: Int) -> List<Int>:
    var out: List<Int> = []
    for i in 0..x: out.push(i)
    out

for x in [5, 2, 0, 3, 7, 1]: print(spread(x), depth(x), upto(x))
