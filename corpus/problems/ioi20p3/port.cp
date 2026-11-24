// Source: IOI 2020 Day 1 (Carnival Tickets), https://dmoj.ca/problem/ioi20p3
// Problem: n (even) colours of m tickets each, values sorted per colour; n, m <= 1500, k <= m
//   rounds. Each round takes one ticket per colour and pays sum |a[i] - b| for the b minimising
//   it. Maximise the total over k rounds and print the allocation (round per ticket, or -1).
// Solution: in each round the n/2 high tickets count + and the low n/2 count -. Start every colour
//   with its k smallest as -; the j-th upgrade of a colour turns one into its next largest +,
//   gaining x[c][m-1-j] + x[c][k-1-j], decreasing in j, so sort and take the best n*k/2.
//   Then each round gives its n/2 colours with the most + left their largest unused ticket and the
//   rest their smallest. O(nk log(nk)).
// original: corpus/submissions/4724579.cpp (grader function find_maximum)
// checker: corpus/problems/ioi20p3/checker.py
// stdin here: n m k, then the n x m ticket values (each row sorted).
// stdout: the prize, then the n x m allocation (round index or -1).
// Bounds: n: 2..=1500; m: 1..=1500; k: 1..=m; x: 0..=1_000_000_000
let n: 2..=1500 = read()
assert n % 2 == 0
let m: 1..=1500 = read()
let k: 1..=m = read()
domain Colour = 0..n
domain Ticket = 0..m
let x: [Colour, Ticket]0..=1_000_000_000 = read()

// every round pays its n/2 high tickets minus its n/2 low ones. Each colour starts with its
// k smallest as low; its j-th upgrade turns one into its j-th largest, and the gain falls with
// j, so the best n*k/2 upgrades over all colours form a valid choice
let ups = [(gain: x[c, Ticket(m - 1 - j)] + x[c, Ticket(k - 1 - j)], c:) for c in Colour for j in 0..k]
let taken = ups.sorted(by: u => desc(u.gain)).take(n * k / 2).collect()
var high: [Colour]0..=k = 0
for u in taken: high[u.c] += 1
print(taken.sum(of: u => u.gain) - sum(x[c, Ticket(j)] for c in Colour for j in 0..k))

// the n/2 colours with the most high tickets left give their largest, the rest their smallest
var left: [Colour]Deque<Ticket> = [Colour] _ => Deque.of(Ticket.collect())
var round: [Colour, Ticket]?Int = None
for r in 0..k:
    for (rank, c) in Colour.sort(by: c => desc(high[c])).items:
        if rank < n / 2:
            high[c] -= 1
            round[c, left[c].pop()!] = r
        else:
            round[c, left[c].pop_front()!] = r
for c in Colour: print([round[c, t] ?? -1 for t in Ticket])
