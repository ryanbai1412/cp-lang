// NOIP 2020 String Matching (CP Lang 0.2.2): count the ways to write s as
// (AB)^k C with A, B, C non-empty and k >= 1, where odd-count letters in A <= C.
// original: corpus/submissions/5445673.cpp
use std.strings.{z_function}

domain Letter = 'a'..='z'

@multitest
let s: Str = read()
let n = s.len
let z = z_function([c for c in s])

var odd_from: [0..=n]Int = 0
var odd_to: [0..=n]Int = 0
var odd = Marks<Letter>()
for i in (0..n).rev:
    let c = Letter(s[i])
    if c in odd: odd.remove(c) else: odd.add(c)
    odd_from[i] = odd.len
odd.clear()
for i in 0..n:
    let c = Letter(s[i])
    if c in odd: odd.remove(c) else: odd.add(c)
    odd_to[i + 1] = odd.len

var at_most: [0..=26]Int = 0
var ways = 0
for w in 2..n:
    for d in odd_to[w - 1]..=26: at_most[d] += 1
    let copies = min(1 + z[w] / w, (n - 1) / w)
    let (odd_k, even_k) = ((copies + 1) / 2, copies / 2)
    ways += odd_k * at_most[odd_from[w]]
    if even_k > 0: ways += even_k * at_most[odd_from[2 * w]]
print(ways)
