// Codeforces 2157 I: Hyper Smawk Bros (dream).
// A move is good if it kills, lands on a lost position, or pins: lands where the
// only good move is the one just made, which the ban forbids. For even m the
// lost positions are the multiples of m+1. For odd m they still come m+1 or m+2
// apart, m+2 exactly when last+m+1 wins; that question looks only at the two
// nearest lost positions and halves its distance to them per recursive pin, so
// each gap costs O(log m), and the list is built once per m.
const H = 1e6

fn lost_positions(m: Int) -> List<Int>:          // odd m
    var lost = [0]
    let mid = (m + 1) / 2
    fn wins(h: Int, ban: ?Int) -> Bool:
        if h < m: return h > 0
        let p = lost.floor_index(h)!
        fn pin(q: Int) -> Bool:
            let x = (h - q) / 2
            (h - q) % 2 == 0 and x != ban and 1 <= x <= m
                and lost.floor(h - x) == q and not wins(h - x, x)
        // lost[p] - 1 sits m+1 past lost[p-1], so its only good move is mid
        let pin_below = ban != mid and p > 0 and lost[p] - lost[p - 1] == m + 2 and h - mid == lost[p] - 1
        pin_below or pin(lost[p]) or (p > 0 and pin(lost[p - 1]))
    while lost.last <= H:
        lost.push(lost.last + if wins(lost.last + m + 1, ban: none): m + 2 else: m + 1)
    lost

let lost_for = memo(lost_positions)

@!multitest
let (n, m): (1..=H, 2..=1e6) = read()
let lost = if m % 2 == 0: n % (m + 1) == 0 else: lost_for(m).floor(n) == n
print(if lost: "NO" else: "YES")
