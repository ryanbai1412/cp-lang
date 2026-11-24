// n: 256
use std.strings.{RollingHash, RollingHash61, DynamicHash, Hash2D, HashVal}
let n: Int = read()
domain R = 0..n / 16
domain C = 0..16
let ss = [Str([Char(Int('a') + (i * 7919 + i / 3) % 3) for i in 0..n]) for _ in 0..2]
var acc = 0
for r in 0..2048:
    acc += RollingHash(ss[r % 2]).len // timed: strings.RollingHash.new
for r in 0..2048:
    acc += RollingHash61(ss[r % 2]).len // timed: strings.RollingHash61.new
for r in 0..256:
    acc += DynamicHash(ss[r % 2]).len // timed: strings.DynamicHash.new
for r in 0..1024:
    let s = ss[r % 2]
    acc += Hash2D<R, C>((x, y) => Int(s[x.ord * 16 + y.ord])).rect(R(0)..R(1), C(0)..C(1)).h1 // timed: strings.Hash2D.new [fixed: C]
for r in 0..2048:
    let s = ss[r % 2]
    let h = RollingHash(ss[r % 2])
    acc += h.hash_of(s).h1 // timed: strings.RollingHash.hash_of
    let parts = [(hash: h.get(8 * q..8 * q + 8), len: 8) for q in 0..n / 8]
    for _ in 0..4:
        acc += h.concat(parts).h1 // timed: strings.RollingHash.concat
for r in 0..512:
    let s = ss[r % 2]
    var g = RollingHash("")
    for c in s:
        g.push(c) // timed: strings.RollingHash.push
    for _ in s:
        g.pop() // timed: strings.RollingHash.pop
    var g61 = RollingHash61("")
    for c in s:
        g61.push(c) // timed: strings.RollingHash61.push
    for _ in s:
        g61.pop() // timed: strings.RollingHash61.pop
let h = RollingHash(ss[0])
let h61 = RollingHash61(ss[0])
var dh = DynamicHash(ss[0])
let h2 = Hash2D<R, C>((x, y) => Int(ss[0][x.ord * 16 + y.ord]))
let a = h.get(0..4)
let a61 = h61.get(0..4)
for i in 0..1 << 18:
    let j = i * 7 % n
    let k = (i * 13 + 5) % n
    let lo = min(j, k)
    let hi = max(j, k) + 1
    acc += HashVal(i, j).hash_token // timed: strings.HashVal.hash_token
    acc += h.len // timed: strings.RollingHash.len
    if h.is_empty: acc += 1 // timed: strings.RollingHash.is_empty
    acc += h.all.h1 // timed: strings.RollingHash.all
    acc += h.slice(lo, hi).h1 // timed: strings.RollingHash.slice
    acc += h.get(lo..hi).h2 // timed: strings.RollingHash.get
    acc += h.combine(a, h.get(lo..hi), hi - lo).h1 // timed: strings.RollingHash.combine
    acc += h.lcp(j, k) // timed: strings.RollingHash.lcp
    acc += h.lcs(j, k) // timed: strings.RollingHash.lcs
    acc += h61.len // timed: strings.RollingHash61.len
    if h61.is_empty: acc += 1 // timed: strings.RollingHash61.is_empty
    if h61.all == a61: acc += 1 // timed: strings.RollingHash61.all
    if h61.get(lo..hi) == a61: acc += 1 // timed: strings.RollingHash61.get
    if h61.combine(a61, h61.get(lo..hi), hi - lo) == a61: acc += 1 // timed: strings.RollingHash61.combine
    acc += h61.lcp(j, k) // timed: strings.RollingHash61.lcp
    acc += h61.lcs(j, k) // timed: strings.RollingHash61.lcs
    acc += dh.len // timed: strings.DynamicHash.len
    if dh.is_empty: acc += 1 // timed: strings.DynamicHash.is_empty
    acc += dh.all.h1 // timed: strings.DynamicHash.all
    acc += dh.get(lo..hi).h1 // timed: strings.DynamicHash.get
    acc += dh.rev_get(lo..hi).h2 // timed: strings.DynamicHash.rev_get
    if dh.is_palindrome(lo..hi): acc += 1 // timed: strings.DynamicHash.is_palindrome
    dh.set(j, Char(Int('a') + i % 3)) // timed: strings.DynamicHash.set
    acc += h2.rect(R(i % (n / 16))..R(n / 16 - 1), C(i % 16)..C(15)).h1 // timed: strings.Hash2D.rect
print(acc)
