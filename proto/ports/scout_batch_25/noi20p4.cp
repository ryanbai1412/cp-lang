// NOI '20 P4 - Dish Making (DMOJ: noi20p4)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// n ingredients, d[i] grams of each, to be used up completely by m dishes of
// exactly k grams, a dish using one or two ingredients (whole grams).
// sum(d) = m * k and m >= n - 2. Print any plan or -1. T <= 10 cases.
//
// m >= n - 1: sort by amount. The smallest has d < k (or everything is k), so
// top it up from the largest, which always has enough; one ingredient is gone
// and m - (n - 1) drops by nothing. When m > n - 1 the largest has d >= k:
// make a dish of it alone first.
// m = n - 2: split the ingredients into two groups that are each "m = n - 1":
// a subset S with sum(d[i] - k for i in S) = -k. That is a subset sum over
// offsets in [-n*k, n*k], done with a bitset per prefix for reconstruction.
use std.bitset.{Bitset}

@cases
fn case():
    let (n, m, k): (1..=500, 1..=5000, 1..=5000) = read()
    domain I = 0..n
    let d: [I]Int = read()

    type Dish = List<(I, Int)>                       // one or two (ingredient, grams)
    var plan: List<Dish> = []

    // Uses up `group` with exactly group.len - 1 + extra dishes.
    fn greedy(group: List<I>, dishes: Int):
        var left = sorted([(d[i], i) for i in group])
        for made in 0..dishes:
            let (small, i) = left.first!
            let (big, j) = left.last!
            if dishes - made > left.len - 1 or left.len == 1:
                // more dishes than merges left: a dish of the largest alone
                left.pop()
                plan.push([(j, k)])
                if big > k: left.insert_sorted((big - k, j))
            else:
                left.remove_first()
                left.pop()
                plan.push([(i, small), (j, k - small)])
                if big - (k - small) > 0: left.insert_sorted((big - (k - small), j))

    if m >= n - 1:
        greedy(list(I), m)
    else:
        domain Off = -n * k..=n * k               // WISH Q8: a bitset over a signed domain
        var reach: [0..=n]Bitset<Off> = Bitset()
        reach[0].insert(Off(0))
        for i in I:
            reach[i.ord + 1] = reach[i.ord] | reach[i.ord].shifted(d[i] - k)
        if not reach[n].contains(Off(-k)):
            print(-1)
            return
        var (S, rest, off): (List<I>, List<I>, Off) = ([], [], Off(-k))
        for i in I.rev:
            if reach[i.ord].contains(off): rest.push(i)
            else:
                S.push(i)
                off -= d[i] - k
        greedy(S, S.len - 1)
        greedy(rest, rest.len - 1)

    for dish in plan:
        print(*(x for (i, g) in dish for x in (i + 1, g)))

@checker
fn valid(out) -> Bool:
    // every dish k grams of <= 2 distinct ingredients, every ingredient used up
    true
