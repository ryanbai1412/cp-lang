// IOI 2009 Day 2 P1, Garage
// original: corpus/submissions/5106592.cpp
let n: 1..=100 = read()
let m: 1..=2000 = read()
domain Spot = 1..=n
domain Car = 1..=m
let rate: [Spot]1..=100 = read()
let weight: [Car]1..=10_000 = read()

var free = MinHeap<Spot>()
for s in Spot: free.push(s)
var waiting: Deque<Car> = []
var parked: [Car]?Spot = none
var revenue = 0
for _ in 0..2 * m:
    let event: -m..=m = read()
    if event > 0: waiting.push(Car(event))
    else: free.push(parked[Car(-event)]!)
    // the car at the front of the queue takes the lowest free spot
    while not free.is_empty and let car = waiting.pop_front():
        parked[car] = free.pop()
        revenue += rate[parked[car]!] * weight[car]
print(revenue)
