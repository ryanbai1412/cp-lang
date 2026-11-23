let n: 1..=20 = read()
let x: Int = read()
for i in 0..n:
    if (x >> i) & 1 == 1:
        print(i)
