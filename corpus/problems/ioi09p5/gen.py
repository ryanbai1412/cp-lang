#!/usr/bin/env python3
# usage: gen.py seed [size] | --max; n spaces, m cars, then 2m events where a car only leaves
# once it has parked.
import random, sys
MAX = sys.argv[1] == "--max"  # 100 spaces, 2000 cars
r = random.Random(1 if MAX else int(sys.argv[1]))
n = 100 if MAX else r.randint(1, 4)
m = 2000 if MAX else r.randint(1, 8)
print(n, m)
for _ in range(n): print(r.randint(1, 100))
for _ in range(m): print(r.randint(1, 10000))
arrive = list(range(1, m + 1)); r.shuffle(arrive)
free, queue, parked, events = n, [], [], []
while arrive or queue or parked:
    if arrive and (not parked or r.random() < 0.5):
        queue.append(arrive.pop()); events.append(queue[-1])
    else:
        car = parked.pop(r.randrange(len(parked))); events.append(-car); free += 1
    while queue and free:
        parked.append(queue.pop(0)); free -= 1
for e in events: print(e)
