// Source: IOI 2009 P5 (Garage), https://dmoj.ca/problem/ioi09p5
// Problem: a garage has n <= 100 spaces with rates R_s; m <= 2000 cars with weights W_k arrive
//   and leave in a given order of 2m events. An arriving car takes the lowest free space, or
//   waits in a FIFO queue until one frees up, and pays R_s * W_k. Print the total revenue.
// Solution: simulate with a MinHeap of free spaces and a list of waiting cars with a head
//   index; after each event, park cars from the queue front while spaces are free. O(m log n).
// original: corpus/submissions/5106592.cpp
let n: 1..=100 = read()
let m: 1..=2000 = read()
domain Spot = 1..=n
domain Car = 1..=m
let rate: [Spot]1..=100 = read()
let weight: [Car]1..=10_000 = read()

var free = MinHeap<Spot>()
for s in Spot: free.push(s)
var spots = n
var waiting: List<Car> = []
var head = 0
var parked: [Car]Spot = Spot.first!
var revenue = 0
for _ in 0..2 * m:
    let event: -m..=m = read()
    if event > 0: waiting.push(Car(event))
    else:
        free.push(parked[Car(-event)])
        spots += 1
    // the car at the front of the queue takes the lowest free spot
    while head < waiting.len and spots > 0:
        let car = waiting[head]
        head += 1
        spots -= 1
        parked[car] = free.pop()!
        revenue += rate[parked[car]] * weight[car]
print(revenue)
