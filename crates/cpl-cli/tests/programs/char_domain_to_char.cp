// `Char(l)` turns a position of a `Char` domain back into its char; `Int(l)` is its code.
domain Letter = 'a'..='z'
let l = Letter('c')
print(Char(l), Int(l), l.ord)
var s = ""
for x in Letter:
    if x.ord < 3: s.push(Char(x))
print(s, Char(Letter('z')))
