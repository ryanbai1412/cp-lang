// NOIP 2020 String Matching: count the ways to write s as (AB)^k C with A, B
// and C not empty and k >= 1, where the number of letters that occur an odd
// number of times in A is at most that number for C. For each of t strings.
//
// Fix the length w of AB. (AB)^k is a prefix of s exactly when s agrees with
// itself shifted by w for (k - 1) * w letters, which is what the Z-function
// measures. Removing two copies of AB leaves C with the same odd letters, so
// only the parity of k matters: C is s[w..] for odd k and s[2w..] for even
// k. Count the choices of A (a proper prefix of AB) with a table of how many
// prefixes have each number of odd letters.
// original: corpus/submissions/5445673.cpp
use std.strings.{z_function}

domain Letter = 'a'..='z'

let t: Int = read()
for _ in 0..t:
    let s: Str = read()
    let n = s.len
    let z = z_function([c for c in s])

    // odd_from[i]: letters with an odd count in s[i..]; odd_to[i]: in s[..i].
    var odd_from = List.filled(n + 1, 0)
    var odd_to = List.filled(n + 1, 0)
    var seen: [Letter]Bool = false
    for i in (0..n).rev:
        let c = Letter(s[i])
        seen[c] = not seen[c]
        odd_from[i] = odd_from[i + 1] + (if seen[c]: 1 else: -1)
    seen.fill(false)
    for i in 0..n:
        let c = Letter(s[i])
        seen[c] = not seen[c]
        odd_to[i + 1] = odd_to[i] + (if seen[c]: 1 else: -1)

    // at_most[d]: how many of the prefixes s[..1], ..., s[..w - 1] have at most d odd letters.
    var at_most = List.filled(27, 0)
    var ways = 0
    for w in 2..n:
        for d in odd_to[w - 1]..=26: at_most[d] += 1
        // k copies fit, with C not empty
        let copies = min(1 + z[w] / w, (n - 1) / w)
        let (odd_k, even_k) = ((copies + 1) / 2, copies / 2)
        ways += odd_k * at_most[odd_from[w]]
        if even_k > 0: ways += even_k * at_most[odd_from[2 * w]]
    print(ways)
