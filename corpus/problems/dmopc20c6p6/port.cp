// Source: DMOPC 2020 Contest 6 P6 Buckets, https://dmoj.ca/problem/dmopc20c6p6
// Problem: interactive: 2 <= N <= 4e4 full buckets of unknown capacities in 1..=M, M <= 1e4. Within
//   1e5 operations (empty i, fill i, dump i into j, answered with how much j rose) find every
//   capacity.
// Solution: measure one empty bucket cur at a time: pour the next unmeasured full bucket into it
//   (the probe), then top it up from a bucket known to be full (the last one measured, else the
//   largest known one refilled). While cur still takes water the probe gave all of it, so its
//   capacity is what it gave; once cur takes nothing or holds M, its capacity is the total in it
//   and it is the next full source, and the probe it overflowed, now empty, is the next cur.
//   O(N) operations.
// original: corpus/submissions/5200785.cpp
let wire = Wire()
let n: 2..=4e4 = read()
let m: 1..=1e4 = read()
domain Bucket = 1..=n

fn empty(b: Bucket) = print("E", b)
fn fill(b: Bucket) = print("F", b)
fn dump(from: Bucket, into: Bucket) -> 0..=m = wire.ask("D {from} {into}")

var cap: [Bucket]?1..=m = None
var largest: ?Bucket = None
fn measured(b: Bucket, c: 1..=m):
    cap[b] = c
    if largest == None or c > cap[largest!]!: largest = b

fn unmeasured_from(b: ?Bucket) -> ?Bucket:
    var at = b
    while let x = at and cap[x] != None: at = x.next
    at

empty(Bucket.first!)
var full: ?Bucket = None
var probe = Bucket.first
while let cur = unmeasured_from(probe):
    probe = cur.next
    var held = 0
    loop:
        probe = unmeasured_from(probe)
        var gave: ?Int = None
        if let p = probe:
            let x = dump(p, cur)
            if x == 0: break
            held += x
            gave = x
        if held == m: break
        let src = full ?? largest ?? Bucket(2)
        if full == None: fill(src)
        full = None
        let topped = dump(src, cur)
        held += topped
        if topped == 0 or held == m: break
        if let p = probe and let x = gave:
            measured(p, x)
            probe = p.next
    measured(cur, held)
    full = cur

print("A", [cap[b]! for b in Bucket])
