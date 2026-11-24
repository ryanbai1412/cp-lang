// YAC 6 P3 (interactive): two hidden cells in an n×m grid; a query returns the distance to the
// nearer one. Four corner queries give x+y and x−y bounds for each; one more query picks the
// right pairing.
let n: 1..=1e9 = read()
let m: 1..=1e9 = read()
domain X = 1..=n
domain Y = 1..=m
fn near(x: X, y: Y) -> 0..=2e9 = ask("?", x, y)

// each corner's answer is a linear form of the nearer mine: x + y, x - y, y - x or -(x + y)
let a = near(X.first!, Y.first!) + 2
let b = near(X.last!, Y.first!) - n + 1
let c = near(X.first!, Y.last!) - m + 1
let d = n + m - near(X.last!, Y.last!)

// guess that (1, 1) and (n, 1) see the same mine, and probe where that puts it
fn is_mine(x: Int, y: Int) -> Bool:
    if let px = X.at(x):
        if let py = Y.at(y): return near(px, py) == 0
    false
let y = (a + b) / 2
if (a + b) % 2 == 0 and is_mine(a - y, y): print("!", a - y, y, d - (d - c) / 2, (d - c) / 2)
else: print("!", (a + c) / 2, a - (a + c) / 2, (d - b) / 2, d - (d - b) / 2)
