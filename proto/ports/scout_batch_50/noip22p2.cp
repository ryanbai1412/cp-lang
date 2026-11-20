// NOIP 2022 P2, clear a deck of paired cards through n stacks with "place" and "match two bottoms"
// original: corpus/submissions/5445681.cpp
@multitest
let (n, m, k): (1..=300, 1..=2e6, 1..=2 * n) = read()
domain S = 1..=n
domain Card = 1..=k
let deck: [0..m]Card = read()

// leaves_at[i]: when the card dealt at time i is cancelled by its twin, for the first of each pair
var leaves_at: [0..m]?Int = none
var waiting: [Card]?Int = none
for (i, c) in deck.items:
    if let first = waiting[c].take(): leaves_at[first] = i
    else: waiting[c] = i

enum Op = Place(s: S) | Match(s: S, t: S)
var stacks: [S]Deque<Card> = []          // bottom first
var leaves: [Card]Int = 0                // per card now in a stack
var home: [Card]S = S.first
var script: List<Op> = []
var (empty, single, ripe) = (Set<S>(S), Set<S>(), Set<S>())   // ripe: two cards, the bottom leaves first

fn refresh(s: S):
    for pool in [mut empty, mut single, mut ripe]: pool.remove(s)
    match stacks[s].len:
        0 => empty.add(s)
        1 => single.add(s)
        2 => if leaves[stacks[s][0]] < leaves[stacks[s][1]]: ripe.add(s)
        _ => {}

fn place(s: S, card: Card):
    script.push(Place(s))
    if stacks[s].last == card: stacks[s].pop()
    else:
        stacks[s].push(card)
        home[card] = s
    refresh(s)

fn match_bottoms(s: S, t: S):
    script.push(Match(s, t))
    for u in [s, t]:
        stacks[u].pop_front()
        refresh(u)

for (now, card) in deck.items:
    if let twin = leaves_at[now]:
        leaves[card] = twin
        // no spare stack: sit on the lone card that stays longest; otherwise
        // keep the last spare stack free for as long as a lone or ripe one will do
        let s = if empty.is_empty: max_by(single, s => leaves[stacks[s][0]])!
                elif empty.len > 1: empty.first!
                else: single.first ?? ripe.first ?? empty.first!
        place(s, card)
    elif stacks[home[card]].last == card: place(home[card], card)
    else:
        // it is a bottom card: cancel it through the spare stack
        let spare = empty.first!
        place(spare, card)
        match_bottoms(home[card], spare)

print(script.len)
for op in script:
    match op:
        Place(s) => print(1, s)
        Match(s, t) => print(2, s, t)
