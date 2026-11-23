// CF 2003C Turtle and Good Pairs
// in: t; per test n, s. out: a reordering of s maximizing good pairs.
domain Letter = 'a'..='z'

@!multitest
let n = read<2..=2e5>()
let s: Str = read()
var left: [Letter]Int = 0
for c in s: left[Letter(c)] += 1
var out = ""
while out.len < n:
    for l in Letter:
        if left[l] > 0:
            out.push(Char(l))
            left[l] -= 1
print(out)
