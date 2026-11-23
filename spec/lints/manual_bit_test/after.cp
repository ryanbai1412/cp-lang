let n: 1..=20 = read()
let x: Int = read()
for i in 0..n:
    if x.bit(i):
        print(i)
