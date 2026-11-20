// NOIP 2022 P2, clear a deck of paired cards through n stacks with "place" and "match two bottoms" (buildable today-form)
// original: corpus/submissions/5445681.cpp
// The three stack pools are plain Set<Int> refreshed by hand and the script
// is a List of (kind, a, b) printed line by line: the prototype has no
// print of a tuple without its unused field and no Set<(size, stack)>.first_ge.
let tests: 1..=10 = read()
for _ in 0..tests:
    let n: 1..=300 = read()
    let m: 1..=2e6 = read()
    let k: 1..=2 * n = read()
    let deck: [0..m]Int = read()

    // leaves_at[i]: when the card dealt at time i is cancelled by its twin, for the first of each pair
    var leaves_at: [0..m]?Int = none
    var waiting: [1..=k]?Int = none
    for i in 0..m:
        if let first = waiting[deck[i]]:
            leaves_at[first] = i
            waiting[deck[i]] = none
        else: waiting[deck[i]] = i

    var stacks: [0..n]List<Int> = []  // bottom first
    var leaves: [1..=k]Int = 0        // per card value now in a stack
    var home: [1..=k]Int = 0
    var script: List<(Int, Int, Int)> = []
    var empty = Set<Int>()
    var single = Set<Int>()
    var ripe = Set<Int>()  // two cards, and the bottom one leaves first
    for s in 0..n: empty.insert(s)

    fn refresh(s: Int):
        empty.remove(s)
        single.remove(s)
        ripe.remove(s)
        if stacks[s].len == 0: empty.insert(s)
        elif stacks[s].len == 1: single.insert(s)
        elif stacks[s].len == 2 and leaves[stacks[s][0]] < leaves[stacks[s][1]]: ripe.insert(s)

    fn place(s: Int, card: Int):
        script.push((1, s + 1, 0))
        if stacks[s].last() == card: stacks[s].pop()
        else:
            stacks[s].push(card)
            home[card] = s
        refresh(s)

    fn match_bottoms(s: Int, t: Int):
        script.push((2, s + 1, t + 1))
        for u in [s, t]:
            var rest: List<Int> = []
            for i in 1..stacks[u].len: rest.push(stacks[u][i])
            stacks[u] = rest
            refresh(u)

    for now in 0..m:
        let card = deck[now]
        if let twin = leaves_at[now]:
            leaves[card] = twin
            if empty.len == 0:
                // no spare stack: sit on the lone card that stays longest
                var best = single.min()!
                for s in 0..n:
                    if stacks[s].len == 1 and leaves[stacks[s][0]] > leaves[stacks[best][0]]: best = s
                place(best, card)
            elif empty.len > 1: place(empty.min()!, card)
            elif let s = single.min(): place(s, card)  // keep the last spare stack free
            elif let s = ripe.min(): place(s, card)
            else: place(empty.min()!, card)
        else:
            let s = home[card]
            if stacks[s].last() == card: place(s, card)
            else:
                // it is a bottom card: cancel it through the spare stack
                let spare = empty.min()!
                place(spare, card)
                match_bottoms(s, spare)

    print(script.len)
    for op in script:
        if op.0 == 1: print(op.0, op.1)
        else: print(op.0, op.1, op.2)
