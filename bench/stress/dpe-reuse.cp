// AtCoder DP E Knapsack 2 (proto/ports/dpe.cp), with a planted bug: the totals run upwards, so an
// item can be taken twice.
// class: mutation
let (n, cap): (1..=100, 1..=1e9) = read()
let items: [0..n](w: 1..=1e9, v: 1..=1000) = read()

fn best(items: List<(w: Int, v: Int)>) -> Int:
    let top = sum(v for (_, v) in items)
    var lightest: List<?Int> = List.filled(top + 1, None)
    lightest[0] = 0
    for (w, v) in items:
        for total in v..=top: lightest[total] min= lightest[total - v] + w
    [total for total in 0..=top if lightest[total] != None and lightest[total]! <= cap].max()!

@brute(best)
fn best_slow(items: List<(w: Int, v: Int)>) -> Int:
    var top = 0
    for mask in 0..(1 << items.len):
        var (w, v) = (0, 0)
        for i in 0..items.len:
            if (mask >> i) & 1 == 1:
                w += items[i].w
                v += items[i].v
        if w <= cap: top max= v
    top

print(best(items))
