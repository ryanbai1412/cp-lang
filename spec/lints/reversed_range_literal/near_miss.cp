// A range whose start can be at or below its end may run.
let (n, k): (1..=10, 0..=10) = read()
for i in k..n:
    print(i)
for i in n..n:
    print(i)
