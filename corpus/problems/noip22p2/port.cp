// Source: NOIP 2022 Meow Meow (Cards and Stacks), https://dmoj.ca/problem/noip22p2
// Problem: T tests, total m <= 2e6: n <= 300 empty stacks and a deck of m cards with k <= 2n - 1
//   patterns, each appearing an even number of times. Moves: put the top deck card on
//   a stack (two equal cards on top cancel), or cancel two equal bottom cards of two stacks. Print
//   a plan of m..=2m moves that clears everything (one is guaranteed).
// Solution: simulate the deck greedily, knowing when each card's twin comes. A first card goes
//   on a fresh empty stack while two are empty, else on a lone card, else on a two-card stack
//   whose bottom leaves first, else on the last spare; with no empty stack it sits on the lone
//   card that stays longest. A twin cancels on top, or through the spare stack when its card is
//   at a bottom. Bitsets of empty, single and ripe stacks; a stack holds the deal times of its cards, so a
//   card knows its twin. O(m log n).
// original: corpus/submissions/5445681.cpp
enum Move = Put(s: Stack) | Pair(s: Stack, t: Stack)

@!multitest
let (n, m) = read<(1..=300, 1..=2e6)>()
let k = read<1..=2 * n - 1>()
domain Stack = 1..=n
domain Card = 1..=k
domain Time = 0..m
let deck = read<[Time]Card>()

// twin[t]: when the card dealt at t is cancelled, for the first card of each pair
var twin: [Time]?Time = None
var unmatched: [Card]?Time = None
for t in Time:
    if let first = unmatched[deck[t]]:
        twin[first] = t
        unmatched[deck[t]] = None
    else: unmatched[deck[t]] = t

var stacks: [Stack]Deque<Time> = [Stack] _ => Deque()  // deal times, bottom first
var home: [Card]?Stack = None
var script: List<Move> = []
var empty = Bitset<Stack>.new()
var single = Bitset<Stack>.new()
var ripe = Bitset<Stack>.new()  // two cards, and the bottom one leaves first
for s in Stack: empty.add(s)

fn top(s: Stack) -> ?Card = stacks[s].last.map(t => deck[t])
fn leaves(t: Time) -> Time = twin[t]!

fn refresh(s: Stack):
    empty.remove(s)
    single.remove(s)
    ripe.remove(s)
    let cards = stacks[s]
    if cards.len == 0: empty.add(s)
    elif cards.len == 1: single.add(s)
    elif cards.len == 2 and leaves(cards.first!) < leaves(cards.last!): ripe.add(s)

fn place(s: Stack, t: Time):
    script.push(Put(s))
    if top(s) == Some(deck[t]): stacks[s].pop()
    else:
        stacks[s].push(t)
        home[deck[t]] = s
    refresh(s)

fn match_bottoms(s: Stack, spare: Stack):
    script.push(Pair(s, spare))
    for u in [s, spare]:
        stacks[u].pop_front()
        refresh(u)

for now in Time:
    if twin[now] != None:
        let s = if empty.len == 0: single.each().max(by: s => leaves(stacks[s].first!))!  // stays longest
            elif empty.len > 1: empty.first()!
            else: single.first() ?? ripe.first() ?? empty.first()!  // keep the last spare stack free
        place(s, now)
    else:
        let s = home[deck[now]]!
        if top(s) == Some(deck[now]): place(s, now)
        else:
            // it is a bottom card: cancel it through the spare stack
            let spare = empty.first()!
            place(spare, now)
            match_bottoms(s, spare)

print(script.len)
for mv in script:
    match mv:
        Put(s) => print(1, s)
        Pair(s, t) => print(2, s, t)
