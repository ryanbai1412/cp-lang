// Loops, labels, ranges with steps and both directions.
for i in 0..5:
    if i == 1:
        continue
    if i == 4:
        break
    print("i", i)
for i in (0..10).rev:
    if i % 3 == 0:
        print("rev", i)
for i in 0..=10 by 4:
    print("by", i)
for i in (1..=10 by 3).rev:
    print("back", i)
var n = 27
var steps = 0
while n != 1:
    n = if n % 2 == 0: n / 2 else: 3 * n + 1
    steps += 1
print(steps)
var found = (0, 0)
'outer: for i in 1..20:
    for j in i..20:
        if i * j == 91:
            found = (i, j)
            break 'outer
print(found)
var odd_pairs = 0
'rows: for i in 0..6:
    for j in 0..6:
        if j > i:
            continue 'rows
        if (i + j) % 2 == 1:
            odd_pairs += 1
print(odd_pairs)
// The bounds of a loop are fixed when it starts.
var limit = 3
var seen = 0
for i in 0..limit:
    limit += 5
    seen += 1
print(seen, limit)
var xs: List<Int> = [1, 2, 3]
for x in xs:
    if x < 3:
        xs.push(x * 10)
print(xs)
let label = if seen > 2: "many" elif seen > 0: "some" else: "none"
print(label)
var k = 0
while true:
    k += 1
    if k * k > 200:
        break
print(k)
for i in 5..5:
    print("never")
for i in 5..2:
    print("never")
var acc = 0
for i in 0..4:
    for j in 0..4:
        if j == 2:
            break
        acc += i * j
print(acc)
