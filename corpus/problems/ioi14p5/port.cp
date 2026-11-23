// IOI 2014 Day 2 P2, Friend: a network grows one person at a time. Person i
// is brought in by an earlier host and becomes a friend of the host only
// (protocol 0), of the host's friends only (1), or of both (2). Pick people
// with the largest total confidence so that no two picked are friends.
//
// Undo the growth from the last person back, folding each person into the
// host. A person then stands for a group, with two numbers: the best total
// of the group when the person is in the sample and when they are not.
// input: n, the n confidences, then `host protocol` for persons 1..n-1
// original: corpus/submissions/6567598.cpp
let n: Int = read()
domain P = 0..n
let conf: [P]Int = read()
var joins: List<(host: P, how: Int)> = []
for _ in 1..n:
    let j: (P, Int) = read()
    joins.push((host: j.0, how: j.1))

var with_: [P]Int = copy(conf)  // best of the group with its person picked
var without: [P]Int = 0
for u in P.rev:
    if u == P.first: break
    let (h, how) = joins[u.to_idx - 1]
    let either = max(with_[u], without[u])
    if how == 0:  // friends with the host: not both
        (with_[h], without[h]) = (with_[h] + without[u], without[h] + either)
    elif how == 1:  // same friends as the host: both or either
        (with_[h], without[h]) = (
            max(with_[h] + either, without[h] + with_[u]),
            without[h] + without[u],
        )
    else:  // both rules: exactly one of the two, or neither
        (with_[h], without[h]) = (
            max(with_[h] + without[u], without[h] + with_[u]),
            without[h] + without[u],
        )
print(max(with_[P.first!], without[P.first!]))
