// First draft from acceptance run 1 (cold agent, docs only), Codeforces 2272B, timed round.
@!multitest
let n: 1..=2e5 = read()
let top: Str = read()
let bot: Str = read()
let up = top.count(c => c == 'f')
let down = bot.count(c => c == 'f')
fn pairs(row: Str) -> Int:
    var total = 0
    var run = 0
    for c in row:  // expect: missing manual-run-length
        if c == 'f':
            run += 1
        else:
            total += run / 2
            run = 0
    return total + run / 2
let delta = abs(up - down)
let m = if up >= down: pairs(top) else: pairs(bot)
let left = max(0, delta - 2 * m)
print(if delta % 2 == 1: max(1, left) else: left)
