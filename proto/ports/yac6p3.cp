// cp: 0.1 1f59ac0+dirty
// Yet Another Contest 6 P3 (interactive): two treasures are hidden in an
// n by m grid; asking "? x y" returns the distance (|dx| + |dy|) to the
// nearer one. From the four corners we learn the smallest and largest x + y
// and x - y over the two treasures; they pair up in one of two ways, and one
// more question tells which.
let n: 1..=1e9 = read()
let m: 1..=1e9 = read() where n * m >= 2
type Cell = (x: Int, y: Int)

fn nearest(x: Int, y: Int) -> Int:
    let d: 0..=2e9 = ask("?", x, y)
    d
fn cell(sum: Int, diff: Int) -> Cell: ((sum + diff) / 2, (sum - diff) / 2)

let low_sum = nearest(1, 1) + 2             // smallest x + y
let high_diff = n - 1 - nearest(n, 1)       // largest x - y
let low_diff = nearest(1, m) - m + 1        // smallest x - y
let high_sum = n + m - nearest(n, m)        // largest x + y

// either one treasure is lowest in x + y and highest in x - y, or lowest in both
let p = cell(low_sum, high_diff)
let fits = (low_sum + high_diff) % 2 == 0 and p.x in 1..=n and p.y in 1..=m
if fits and nearest(p.x, p.y) == 0:
    let q = cell(high_sum, low_diff)
    print("!", p.x, p.y, q.x, q.y)
else:
    let (p, q) = (cell(low_sum, low_diff), cell(high_sum, high_diff))
    print("!", p.x, p.y, q.x, q.y)
