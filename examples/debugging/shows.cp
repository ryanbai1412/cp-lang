// Each test: the most TV shows you can watch whole, one at a time. Show i
// runs from `start` for `len` minutes; one may start the minute another
// ends. A bug is planted in the greedy order.
@!multitest
let n: 1..=6 = read()
domain I = 1..=n
let shows: [I](start: 0..=9, len: 1..=4) = read()

fn most_watched() -> Int:
    var order = [i for i in I]
    sort(mut order, by: i => shows[i].start)
    var free_at = 0
    var watched = 0
    for i in order:
        if shows[i].start >= free_at:
            watched += 1
            free_at = shows[i].start + shows[i].len
    return watched

print(most_watched())

@alt(program)
fn every_subset():
    var best = 0
    for mask in 0..(1 << n):
        let picked = [i for i in I if (mask >> i.ord) & 1 == 1]
        var fits = true
        for a in picked:
            for b in picked:
                if a != b and shows[a].start < shows[b].start + shows[b].len and shows[b].start < shows[a].start + shows[a].len:
                    fits = false
        if fits: best max= picked.len
    print(best)
