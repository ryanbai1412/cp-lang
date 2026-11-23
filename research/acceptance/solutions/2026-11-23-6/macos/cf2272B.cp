@!multitest
let n: 1..=2e5 = read()
let top: Str = read()
let bottom: Str = read()
print((top.count(c => c == 'f') + bottom.count(c => c == 'f')) % 2)
