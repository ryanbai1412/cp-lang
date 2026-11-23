@!multitest
let n: 1..=2e5 = read()
let top: Str = read()
let bot: Str = read()
let t = top.count(c => c == 'f')
let b = bot.count(c => c == 'f')
let row = if t >= b: top else: bot
let d = abs(t - b)
var m = 0
for r in row.runs():
    if r.first! == 'f':
        m += r.len / 2
print(d - 2 * min(m, d / 2))
