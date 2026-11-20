// Mock CCC '21 S1 (nccc8s1): print every distinct string (other than s) at
// edit distance 1 from s using letters from alphabet al, in sorted order.
// original: corpus/submissions/3381111.java
let (al, s): (Str, Str) = read()
var out = Set<Str>()
for i in 0..s.len:
    out.add(s[..i] + s[i + 1..])
    for c in al:
        out.add("{s[..i]}{c}{s[i + 1..]}")
        out.add("{s[..i]}{c}{s[i..]}")
for c in al:
    out.add("{s}{c}")
out.remove(s)
for w in out: print(w)
