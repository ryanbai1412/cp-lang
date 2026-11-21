type Z = Mod<998_244_353>

struct Low(cover: Int, count: Int)

fn transform(l: Low, f: fn(Low) -> Low) -> Low = f(l)

let n: 1..=20 = read()
var ones, open, ans: Z = 0
for a in read<[n]1..=3>():
    match a:
        1 => ones += 1
        2 => open = 2 * open + ones
        3 => ans += open
print(ans)

let m: 1..=5 = read()
var ways, next: [1..=m]Z = 0
ways[1] = 1
next[m] = 2
print(ways[1], next[m])

let xs = [Low(0, 1), Low(2, 3)]
let updated = transform(xs[0], (l: Low) => l with cover: l.cover + 5)
let both = updated with cover: updated.cover + 1, count: 9
print(updated.cover, updated.count, both.cover, both.count)
