// n: 512
// fixed: A
use std.strings.{PalindromicTree}
domain L = 0..4
let n: Int = read()
let ws = [[L((i * 7919 + i / 3 + j) % 3) for i in 0..n] for j in 0..2]
let half = [L((i * 7919 + i / 3) % 3) for i in 0..n / 2]
let pal = half + half.rev.collect()
let s = Str([Char(Int('a') + (i * 7919 + i / 3) % 3) for i in 0..n])
var acc = 0
let p = manacher(s)
for r in 0..8192:
    acc += p.count() // timed: strings.Palindromes.count
    acc += p.longest().l // timed: strings.Palindromes.longest
for r in 0..1024:
    let w = ws[r % 2]
    let t = PalindromicTree<L>.of(w) // timed: strings.PalindromicTree.of
    acc += t.min_palindrome_partition()[n] // timed: strings.PalindromicTree.min_palindrome_partition
    var g = PalindromicTree<L>()
    for c in w:
        acc += g.push(c) // timed: strings.PalindromicTree.push
    for _ in w:
        g.pop() // timed: strings.PalindromicTree.pop
    for c in w:
        acc += g.push_front(c) // timed: strings.PalindromicTree.push_front
let tp = PalindromicTree<L>.of(pal)
let ta = PalindromicTree<L>.of([L(0) for _ in 0..n])
for r in 0..4096:
    acc += ta.occ_counts().len // timed: strings.PalindromicTree.occ_counts
for r in 0..2048:
    if tp.contains(pal): acc += 1 // timed: strings.PalindromicTree.contains
let t0 = PalindromicTree<L>.of(ws[0])
let states = t0.len
for i in 0..1 << 18:
    let j = i * 7 % n
    let v = 2 + i * 13 % (states - 2)
    let lo = i % n
    let hi = lo + i * 31 % (n - lo + 1)
    acc += PalindromicTree<L>().len // timed: strings.PalindromicTree.new
    acc += p.len // timed: strings.Palindromes.len
    if p.is_empty: acc += 1 // timed: strings.Palindromes.is_empty
    if p.is_palindrome(lo..hi): acc += 1 // timed: strings.Palindromes.is_palindrome
    acc += p.longest_at(j) // timed: strings.Palindromes.longest_at
    acc += p.longest_between(j) // timed: strings.Palindromes.longest_between
    acc += Int(t0.at(j)) // timed: strings.PalindromicTree.at
    acc += t0.diff(v) // timed: strings.PalindromicTree.diff
    acc += t0.distinct // timed: strings.PalindromicTree.distinct
    acc += t0.last_state // timed: strings.PalindromicTree.last_state
    acc += t0.len // timed: strings.PalindromicTree.len
    acc += t0.link(v) // timed: strings.PalindromicTree.link
    acc += t0.pal_len(v) // timed: strings.PalindromicTree.pal_len
    acc += t0.series_link(v) // timed: strings.PalindromicTree.series_link
    if let u = t0.step(v, L(i % 3)): acc += u // timed: strings.PalindromicTree.step
print(acc)
