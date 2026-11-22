for i in 0..5:
    if i == 3:
        break
else:
    print("no break 1")
for i in 0..5:
    if i == 9:
        break
else:
    print("finished 2")
var k = 0
while k < 3:
    k += 1
else:
    print("while done")
for i in 0..3:
    for j in 0..3:
        if j == 1:
            break
    else:
        print("inner")
else:
    print("outer")
let pairs = [(1, Some(10)), (2, None), (3, Some(30))]
for p in pairs:
    let (a, Some(b)) = p else: continue
    print(a + b)
