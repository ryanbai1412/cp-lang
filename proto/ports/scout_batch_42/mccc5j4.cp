// Mock CCC '23 Contest 1 J4 — Lexicographically Smallest Wildcard Fill (CP Lang v2 spec)
// original: corpus/submissions/5333145.cpp
// spec: docs/one-page-cheat-sheet.md (3723dca)
//
// Replace each '*' in lowercase string s with 'a'..'z' (values 1..26) so the total
// letter sum equals k and the resulting string is lexicographically smallest.
// Rethought as a single forward greedy pass with zero auxiliary stack.

let k: Int = read()
var s: Str = read()

var rem = k
var wild = 0
for ch in s:
    if ch == '*': wild += 1
    else: rem -= ch - 'a' + 1

if rem < wild or rem > 26 * wild:
    print("Impossible")
    return

for i in 0..s.len:
    if s[i] == '*':
        wild -= 1
        let d = max(1, rem - 26 * wild)
        // FRICTION MC8: Cheat sheet defines `ch - 'a'` (Char - Char -> Int) and
        // `Char.digit(d)` for decimal digits, but leaves the inverse letter shift
        // `'a' + (d - 1)` (Char + Int -> Char) undocumented in the v2 spec.
        s[i] = 'a' + (d - 1)
        rem -= d

print(s)
