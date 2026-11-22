let n: 1..=9 = read()
domain V = 1..=5
let a: [n]V = read()
var c: Counter<V> = []
for v in a:
    c[v] += 1
    print(v, c[v], c.distinct)
c[V(1)] -= 2
print(c.distinct, c[V(1)], c[V(5)])
let d = counts(a)
print(d.distinct, d[V(2)])
var h: Counter<Int> = []
for v in a: h[v.label * 1000000007] += 1
h[5] = 3
h[5] -= 3
print(h.distinct, h[2000000014], h[7])
let words = counts([(1, 2), (1, 2), (3, 4)])
print(words.distinct, words[(1, 2)], words[(9, 9)])

// a word read straight into a domain of letters
domain Letter = 'a'..='z'
let s: [domain S]Letter = read()
let t: [domain T]Char = read()
let letters = counts(s)
print(letters.distinct, letters[Letter('b')], s.len, t.len, s[S.first], s[S.last])
let hash = StrHash(t)
print(hash.of(..=T.last) == hash.of(T.first, T.last), hash.of((..=T.last).suffix(2)) == hash.of((T.first..).prefix(2)))
