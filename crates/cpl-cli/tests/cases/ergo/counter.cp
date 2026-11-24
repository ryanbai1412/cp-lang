let n: 1..=9 = read()
domain V = 1..=5
let a: [n]V = read()
var c = Counter<V>()
for v in a:
    c[v] += 1
    print(v, c[v], c.len)
c[V(1)] -= 2
print(c.len, c[V(1)], c[V(5)])
let d = counts(a)
print(d.distinct, d[V(2)])
var h = Counter<Int>()
for v in a: h[v.label * 1000000007] += 1
h[5] = 3
h[5] -= 3
print(h.len, h[2000000014], h[7])
let words = counts([(1, 2), (1, 2), (3, 4)])
print(words.distinct, words[(1, 2)], words[(9, 9)])

// a word read straight into a domain of letters
domain Letter = 'a'..='z'
let s: [domain S]Letter = read()
let t: [domain T]Char = read()
let letters = counts(s)
print(letters.distinct, letters[Letter('b')], s.len, t.len, s[S.first!], s[S.last!])
let hash = RollingHash(t)
print(yes(hash.get(..=T.last!) == hash.get(T.first!..=T.last!)), yes(hash.get(T.last!.prev!..=T.last!) == hash.get(T.first!..=T.first!.next!)))
