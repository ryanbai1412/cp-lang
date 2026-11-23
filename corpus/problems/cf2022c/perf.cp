// Codeforces 2022 C, tuned: port.cp's memoized `best` as a table filled from the right, one
// slot per (cut, front), with the votes read once into two Int lists, as orig.cpp's dp does.
@!multitest
let n: 3..=100_000 = read()
let top: Str = read()
let bottom: Str = read()
let v = [[Int(ch == 'A') for ch in top], [Int(ch == 'A') for ch in bottom]]
fn won(a: Int, b: Int, c: Int) -> Int = Int(a + b + c >= 2)
let lost = -1_000_000
var flush = List.filled(n + 1, lost)
var behind = [List.filled(n + 1, lost), List.filled(n + 1, lost)]
flush[n] = 0
for c in (0..n).rev:
    for z in 0..2:
        let o = 1 - z
        if c + 2 <= n:
            var b = won(v[z][c], v[z][c + 1], v[o][c + 1]) + flush[c + 2]
            if c + 4 <= n:
                let bars = won(v[z][c], v[z][c + 1], v[z][c + 2]) + won(v[o][c + 1], v[o][c + 2], v[o][c + 3])
                b = max(b, bars + behind[z][c + 3])
            behind[z][c] = b
    var f = lost
    if c + 3 <= n:
        f = won(v[0][c], v[0][c + 1], v[0][c + 2]) + won(v[1][c], v[1][c + 1], v[1][c + 2]) + flush[c + 3]
    if c + 1 < n:
        for z in 0..2:
            let o = 1 - z
            f = max(f, won(v[z][c], v[o][c], v[o][c + 1]) + behind[z][c + 1])
    flush[c] = f
print(flush[0])
