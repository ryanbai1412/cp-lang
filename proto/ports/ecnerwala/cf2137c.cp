@!multitest
let (a, b): (Int, Int) = read()
if a % 2 == 0 or b % 2 == 0:
    if b % 2 == 0 and (a * b) % 4 == 0:
        print(a * b / 2 + 2)
    else:
        print(-1)
else:
    print(a * b + 1)
