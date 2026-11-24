// Source: IOI 2020 Day 1 (Carnival Tickets), https://dmoj.ca/problem/ioi20p3
// limits: 2000 ms, 512 MB
// Problem: n (even) colours of m tickets each, values sorted per colour; n, m <= 1500, k <= m
//   rounds. Each round takes one ticket per colour and pays sum |a[i] - b| for the b minimising
//   it. Maximise the total over k rounds and print the allocation (round per ticket, or -1).
// Solution: in each round the n/2 high tickets count + and the low n/2 count -. Start every colour
//   with its k smallest as -; the j-th upgrade of a colour turns one into its next largest +,
//   gaining x[c][m-1-j] + x[c][k-1-j], decreasing in j, so a MaxHeap of each colour's next
//   upgrade hands out the best n*k/2 in order.
//   Then each round gives its n/2 colours with the most + left their largest unused ticket and the
//   rest their smallest. O(nk log n).
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

var high: [Colour]0..=k = 0
var pq = MaxHeap<(Int, Colour)>()
fn offer(c: Colour):
    if high[c] < k: pq.push((x[c, Ticket(k - 1 - high[c])] + x[c, Ticket(m - 1 - high[c])], c))
for c in Colour: offer(c)
var prize = -sum(x[c, Ticket(j)] for c in Colour for j in 0..k)
for _ in 0..n * k / 2:
    let (gain, c) = pq.pop()!
    prize += gain
    high[c] += 1
    offer(c)
print(prize)

var low: [Colour]0..=k = [Colour] c => k - high[c]
var round: [Colour, Ticket]-1..1500 = -1
for r in 0..k:
    var lows = 0
    var open: List<Colour> = []
    for c in Colour:
        if low[c] == 0:
            round[c, Ticket(m - high[c])] = r
            high[c] -= 1
        elif high[c] == 0:
            low[c] -= 1
            round[c, Ticket(low[c])] = r
            lows += 1
        else: open.push(c)
    for c in open:
        if lows < n / 2:
            low[c] -= 1
            round[c, Ticket(low[c])] = r
            lows += 1
        else:
            round[c, Ticket(m - high[c])] = r
            high[c] -= 1
for c in Colour: print([round[c, t] for t in Ticket])
