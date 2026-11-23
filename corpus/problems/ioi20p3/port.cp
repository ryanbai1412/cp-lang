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
// stdin here: n m k, then the n x m ticket values (each row sorted).
// stdout: the prize, then the n x m allocation (round index or -1).
let n: 2..=1500 = read()
let m: 1..=1500 = read()
let k: 1..=m = read()
domain Colour = 0..n
domain Ticket = 0..m
let x: [Colour, Ticket]Int = read()

// (-gain, colour, j): colour's j-th upgrade
var ups: List<(Int, Int, Int)> = []
var prize = 0
for c in Colour:
    for j in 0..k:
        prize -= x[c, Ticket(j)]
        ups.push((-(x[c, Ticket(m - 1 - j)] + x[c, Ticket(k - 1 - j)]), c.to_idx, j))
ups.sort()

var plus: [Colour]Int = 0
for i in 0..(n * k / 2):
    let u = ups[i]
    prize -= u.0
    plus[Colour(u.1)] += 1

var round: [Colour, Ticket]Int = -1
var lo: [Colour]Int = 0
var hi: [Colour]Int = m - 1
for r in 0..k:
    var by_plus: List<(Int, Int)> = []
    for c in Colour:
        by_plus.push((-plus[c], c.to_idx))
    by_plus.sort()
    for rank in 0..n:
        let c = Colour(by_plus[rank].1)
        if rank < n / 2 and plus[c] > 0:
            round[c, Ticket(hi[c])] = r
            hi[c] -= 1
            plus[c] -= 1
        else:
            round[c, Ticket(lo[c])] = r
            lo[c] += 1

print(prize)
for c in Colour:
    var row: List<Int> = []
    for t in Ticket:
        row.push(round[c, t])
    print(row)
