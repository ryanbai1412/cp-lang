// Tuned: one prefix-count table, reused across the binary search, with cells as
// narrow as their counts; built in place instead of from a fresh 0/1 grid each check.
let (r, c, h, w): (1..=3000, 1..=3000, 1..=3000, 1..=3000) = read()
let rank: [0..r, 0..c]1..=9_000_000 = read()
var low: [0..=r, 0..=c]0..=9_000_000 = 0

// A window's median is at most x exactly when more than half of its cells are.
fn some_median_at_most(x: Int) -> Bool:
    for i in 0..r:
        for j in 0..c:
            unchecked:
                low[i + 1, j + 1] = low[i + 1, j] + low[i, j + 1] - low[i, j] + Int(rank[i, j] <= x)
    let half = h * w / 2
    for i in 0..=r - h:
        for j in 0..=c - w:
            if low[i + h, j + w] - low[i, j + w] - low[i + h, j] + low[i, j] > half: return true
    false
print((1..=r * c).first_true(some_median_at_most)!)
