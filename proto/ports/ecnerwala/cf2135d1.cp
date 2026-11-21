@!interactive

const WMAX = 100_000
const G = 125
const Q1 = 10_000
const Q02 = 15_000

fn ask(vals: List<Int>) -> Int:
    print("?", vals.len, vals)
    read()

@!multitest
let resp1 = ask(List.filled(Q1, G))
if resp1 == 0:
    let resp2 = ask(List.filled(Q02, 1))
    print("!", first(ans for ans in 1..=Q02 if resp2 == ceil_div(Q02, ans))!)
    return

let w_min = ceil_div(Q1, resp1) * G
let w_max = min(WMAX, (Q1 - 1) / (resp1 - 1) * G + G - 1)
var q: List<Int> = []
for i in w_min + 1..=w_max: q.extend([w_min, i - w_min])
let resp2 = ask(q)
print("!", (w_max - w_min) + w_max - resp2)
