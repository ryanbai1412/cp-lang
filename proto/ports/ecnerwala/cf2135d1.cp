fn ask(vals: List<Int>) -> Int:
    print("?", vals.len, vals)
    flush()
    read()

@!multitest
let wmax = 100_000
let g = 125
let q1 = 10_000
let q02 = 15_000
let resp1 = ask([g for _ in 0..q1])
if resp1 == 0:
    let resp2 = ask([1 for _ in 0..q02])
    var ans = 1
    while resp2 != ceil_div(q02, ans): ans += 1
    print("!", ans)
    flush()
else:
    let wg_min = ceil_div(q1, resp1)
    let wg_max = (q1 - 1) / (resp1 - 1)
    let w_max = min(wmax, wg_max * g + g - 1)
    let w_min = wg_min * g
    var q: List<Int> = []
    for i in (w_min + 1)..=w_max:
        q.push(w_min)
        q.push(i - w_min)
    let resp2 = ask(q)
    print("!", (w_max - w_min) + w_max - resp2)
    flush()
