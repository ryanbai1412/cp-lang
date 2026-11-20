// IOI 2009 Day 1 P4, Garage: n parking spaces with a rate each, m cars
// with a weight each, and 2m events: a positive number is that car
// arriving, a negative one that car leaving. An arriving car takes the
// lowest-numbered free space, or waits in a queue until one frees up.
// Print the total of rate * weight over all cars parked.
// original: corpus/submissions/5106592.cpp
use std.heap.{MinHeap}

let (n, m): (Int, Int) = read()
domain Space = 1..=n
domain Car = 1..=m
let rate: [Space]Int = read()
let weight: [Car]Int = read()
// WISH C7: `var free = Set<Space>.full()` with `free.pop_first()`; the
// interpreter has no ordered set, so a heap of ordinals stands in.
var free = MinHeap<Int>()
for s in Space: free.push(s.to_idx)
var waiting: List<Car> = []
var served = 0                              // cars of `waiting` already parked
var parked: [Car]?Space = none
var takings = 0
for _ in 0..2 * m:
    let event: Int = read()
    if event < 0:
        free.push(parked[Car(-event)]!.to_idx)
    else: waiting.push(Car(event))
    while served < waiting.len and not free.is_empty:
        let car = waiting[served]
        let space = Space.nth(free.pop()!)
        served += 1
        parked[car] = space
        takings += rate[space] * weight[car]
print(takings)
