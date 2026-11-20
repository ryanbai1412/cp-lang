@!multitest
let (n, k): (Int, Int) = read()
if k % 2 == 1:
    print([n for _ in 0..k])
else:
    var a: [0..k]Int = 0
    var num_smaller = 0
    for b in (0..=30).rev:
        if n.bit(b):
            if num_smaller < k: num_smaller += 1
            for i in 0..k:
                if i != num_smaller - 1:
                    a[i] += 1 << b
        else:
            let lim = (num_smaller / 2) * 2
            for i in 0..lim:
                a[i] += 1 << b
    print(a)
