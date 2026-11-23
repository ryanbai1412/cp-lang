@!multitest
let n: 1..=2e5 = read()
let top: Str = read()
let bot: Str = read()
print((top.count(c => c == 'f') + bot.count(c => c == 'f')) % 2)
