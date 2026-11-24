// Source: NOIP 2020 P2 (String Matching), https://dmoj.ca/problem/noip20p2
// limits: 1000 ms, 512 MB
// Problem: for a lowercase string S (|S| <= 2^20), count the ways to write S = (AB)^i C with
//   A, B, C non-empty and i >= 1, where F(A) <= F(C) and F(X) is the number of letters occurring
//   an odd number of times in X. Multitest (T <= 5).
// Solution: fix w = |AB|. The Z-function gives how many times the prefix of length w repeats;
//   C must stay non-empty. (AB)^2 has all counts even, so F(C) is F of the suffix after w for
//   odd i and after 2w for even i. at_most[d] counts prefixes A of length < w with F(A) <= d,
//   grown by one prefix per w. O(26 n).
// original: corpus/submissions/5445673.cpp

domain Letter = 'a'..='z'
domain Odd = 0..=Letter.len

/// F of every prefix of `xs`: element c counts the letters odd in xs[..c].
fn odd_prefixes(xs: List<Letter>) -> List<Odd>:
    var odd = Bitset<Letter>.new()
    var f = [Odd(0)]
    for c in xs:
        odd.toggle(c)
        f.push(Odd(odd.len))
    f

@multitest
let s: Str = read()
let n = s.len
let letters = [Letter(c) for c in s]
let before = odd_prefixes(letters)          // before[c] = F(s[..c])
let after = List(odd_prefixes(letters.rev).rev)  // after[c] = F(s[c..])
let z = z_function(s)

var at_most: [Odd]Int = 0  // prefixes A shorter than w with F(A) <= d
var ways = 0
for w in 2..n:
    for d in before[w - 1]..=Odd.last!: at_most[d] += 1
    let copies = min(1 + z[w] / w, (n - 1) / w)
    ways += (copies + 1) / 2 * at_most[after[w]]
    if copies >= 2: ways += copies / 2 * at_most[after[2 * w]]
print(ways)
