// Blind Critic #4 — First Draft (written from docs/one-page-cheat-sheet.md v2.1 ONLY)
// Problem: ioi11p5 (Dancing Elephants)
// Original: corpus/submissions/3469626.cpp
// Exercises: for mut, struct with mut methods, List<Bucket>, .first_ge, Tropical Optionals, @brute

const B = 400

type Jump = (end: Int, cnt: Int)

struct Bucket:
    xs: List<Int> = []
    dp: List<Jump> = []

    fn rebuild(mut self, l: Int):
        let k = self.xs.len
        self.dp = List.filled(k, (end: 0, cnt: 0))
        var j = k - 1
        for i in (0..k).rev:
            while j >= 0 and self.xs[j] > self.xs[i] + l:
                j -= 1
            if j + 1 == k:
                self.dp[i] = (end: self.xs[i], cnt: 1)
            else:
                let nxt = self.dp[j + 1]
                self.dp[i] = (end: nxt.end, cnt: nxt.cnt + 1)

    fn remove_val(mut self, x: Int, l: Int):
        // BLIND-GUESS(H2): does xs.first_ge(x) return ?Int index or ?Int value?
        // BLIND-GUESS(H1): List.remove(idx) is not listed in cheat sheet §9!
        let idx = self.xs.first_ge(x)!
        self.xs.remove(idx)
        self.rebuild(l)

    fn insert_val(mut self, x: Int, l: Int):
        // BLIND-GUESS(H1): List.insert(idx, x) is not listed in cheat sheet §9!
        let idx = self.xs.first_ge(x) ?? self.xs.len
        self.xs.insert(idx, x)
        self.rebuild(l)

let (n, l, q): (1..=150_000, 0..=1_000_000_000, 1..=150_000) = read()
domain E = 0..n
var pos: [E]Int = read()

var buckets: List<Bucket> = []

fn build_buckets(sorted_xs: List<Int>):
    buckets.clear()
    for chunk in (0..sorted_xs.len).chunks(B):
        // BLIND-GUESS(H4): struct construction syntax not shown in cheat sheet §7!
        var b = Bucket(xs: sorted_xs[chunk].copy())
        b.rebuild(l)
        buckets.push(b)

fn flatten_and_rebuild():
    var all: List<Int> = []
    for b in buckets:
        for x in b.xs:
            all.push(x)
    build_buckets(all)

var init_sorted = [pos[e] for e in E]
init_sorted.sort()
build_buckets(init_sorted)

var ops_since_rebuild = 0

for _ in 0..q:
    let (e, y): (E, 0..=1_000_000_000) = read()
    if ops_since_rebuild == B:
        flatten_and_rebuild()
        ops_since_rebuild = 0
    ops_since_rebuild += 1

    let old = pos[e]
    // Remove old from first bucket whose max >= old
    for mut b in buckets:
        // BLIND-GUESS(H3): if let with trailing boolean guard (cheat sheet shows while let ... and cond, not if let ... and cond)
        if let last = b.xs.last and last >= old:
            b.remove_val(old, l)
            break

    // Insert y into first bucket whose max >= y, else last non-empty bucket
    var placed = false
    var last_nonempty: ?Int = none
    for (bi, x) in buckets.items:
        // HESITATION: can't do `for (bi, mut b) in buckets.items` (cheat sheet only shows `for mut b in blocks`)
        if not buckets[bi].xs.is_empty:
            last_nonempty = bi
            if buckets[bi].xs.last! >= y:
                buckets[bi].insert_val(y, l)
                placed = true
                break
    if not placed:
        let bi = last_nonempty ?? 0
        buckets[bi].insert_val(y, l)

    pos[e] = y

    var cover: ?Int = none
    var ans = 0
    for b in buckets:
        if b.xs.is_empty:
            continue
        // Tropical optional: cover + l + 1 is ?Int (none if cover is none)
        let idx: ?Int = if let need = cover + l + 1: b.xs.first_ge(need) else: 0
        if let k = idx:
            let step = b.dp[k]
            ans += step.cnt
            cover = step.end
    print(ans)

@brute(main)
fn slow():
    var cur = pos.copy()
    // Note: in standalone @brute, would need to replay queries; illustrates read() scoping question
    var tmp = [cur[e] for e in E]
    tmp.sort()
    var cover: ?Int = none
    var ans = 0
    for x in tmp:
        if let c = cover:
            if x > c + l:
                cover = x
                ans += 1
        else:
            cover = x
            ans += 1
    print(ans)
