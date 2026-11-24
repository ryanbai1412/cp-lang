let n: 1..=20 = read()
var boxes = List.filled(1 << n, (l: 0, r: 0))
for i in 0..(1 << n):
    boxes[i] = (l: i * 7919 % 1000, r: i * 104729 % 1_000_003)
boxes.sort()
print(boxes[0].r + boxes[(1 << n) - 1].r)
