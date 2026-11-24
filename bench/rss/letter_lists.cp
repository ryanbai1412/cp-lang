domain Letter = 'a'..='z'
let n: 1..=400_000 = read()
let words = read<[n]Str>()
let lists = [[Letter(c) for c in w] for w in words]
var t = 0
for w in lists:
    for c in w:
        t = (t * 31 + c.ord) % 1_000_000_007
print(t)
