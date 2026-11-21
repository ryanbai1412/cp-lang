var made = 0
var log: List<Int> = []

fn nat() -> yields Int:
    var i = 0
    while true:
        print("make", i)
        made += 1
        log.push(i)
        yield i
        i += 1

fn words() -> yields Int:
    while true:
        let x: Int = read()
        if x == 0: return
        yield x

for x in nat():
    print("got", x, made, log.len)
    if x >= 3: break
print(made, log)
var total = 0
for w in words():
    total += w
    print("word", w)
    if w == 7: continue
    if total > 20: break
let rest: Int = read()
print(total, rest)
