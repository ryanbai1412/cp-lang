@!multitest
let (x, y): (Int, Int) = read()
if x < y:
    print(2)
elif x - y <= 1 or y == 1:
    print(-1)
else:
    print(3)
