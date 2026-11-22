// AAC 5 P4 (today/ runnable subset) — original: corpus/submissions/4239900.cpp

let n: 1..=1_000_000 = read()
let x: 0..=1_000_000_000 = read()

if n == 1:
    print(0)
else:
    var ans = n / 2
    if n % 2 == 1:
        let h = (n - 3) / 2
        let flip = h % 4 == 0 or h % 4 == 3
        if flip != (x % 2 == 1):
            ans += 1
    print(ans)
    for i in 1..=ans:
        print(i)
