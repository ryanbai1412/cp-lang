// Sites join a growing tree one at a time and each new site sends one symbol in
// 0..=4; afterwards the ends of a longest path must be read off the symbols alone.
// Track the diameter online: a new site either becomes an end or changes nothing.
// Stay silent until 21 sites remain. The next 7 spell end u in base 4, one digit
// each, with 4 meaning "u is now me" (the new site always takes u's slot). The last
// 14 spell end v in binary, and say whether the site became v (0), became u
// (1 + bit) or neither (3 + bit). The decoder replays the same story: a "me" symbol
// overwrites an end, any other symbol sets one digit of it.
@grader(two_phase)
mod migrations:
    var tree = GrowingTree()                       // attach and dist in O(log n)
    var u = 0
    var v = 1

    fn extends(i: Int, end: Int) -> Bool = tree.dist(i, end) > tree.dist(u, v)

    fn send_message(n: 2..=1e4, i: 1..n, parent: 0..i) -> 0..=4:
        if i == 1: (tree, u, v) = (GrowingTree(root: 0), 0, 1)
        tree.attach(i, to: parent)
        let left = n - i
        if left > 21:
            if extends(i, u): v = i
            if extends(i, v): u = i
            return 0
        if left > 14:
            if extends(i, u): (u, v) = (i, u); return 4
            if extends(i, v): u = i; return 4
            return u.digit(left - 15, base: 4)
        let bit = v.digit(left - 1, base: 2)
        if extends(i, u): v = i; return 0
        if extends(i, v): u = i; return 1 + bit
        return 3 + bit

    fn longest_path(s: [N]0..=4) -> (N, N):
        var u, v: N = 0
        for i in N.after(0):
            let left = N.count - i
            if left > 21: continue
            if left > 14:
                u = if s[i] == 4: i else: u.with_digit(left - 15, s[i], base: 4)
            else: match s[i]:
                0 => v = i
                1 | 2 => (u, v) = (i, v.with_digit(left - 1, s[i] - 1, base: 2))
                _ => v = v.with_digit(left - 1, s[i] - 3, base: 2)
        return (u, v)
