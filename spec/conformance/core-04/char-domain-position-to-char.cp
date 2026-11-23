// expect: ok, output "c 99 2\nabc"
// rule: 04.2.24
// A position of a `Char` domain converts back with `Char(l)`; `Int(l)` is its code, `l.ord` its ordinal (rebuild round 15 H-LETTER-TO-CHAR).
domain Letter = 'a'..='z'
let l = Letter('c')
print(Char(l), Int(l), l.ord)
var s = ""
for x in Letter:
    if x.ord < 3: s.push(Char(x))
print(s)
