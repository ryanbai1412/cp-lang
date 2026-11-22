// EGOI '22 P2 - Lego Wall (runnable on proto/cp.py lib-run today)

type Z = Mod<1000000007>

let (w, h): (Int, Int) = read()

var row = List.filled(w + 1, Z.zero)
row[0] = 1
row[1] = 1
for i in 2..=w:
    row[i] = row[i - 1] + row[i - 2]

var total = List.filled(w + 1, Z.zero)
total[0] = 1
for i in 1..=w:
    total[i] = row[i] ** h

var solid = List.filled(w + 1, Z.zero)
for i in 1..=w:
    var cur = total[i]
    for j in 1..i:
        cur -= solid[j] * total[i - j]
    solid[i] = cur

print(solid[w])
