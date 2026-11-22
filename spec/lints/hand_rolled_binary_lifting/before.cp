var up = [[0, 0, 1], [0, 0, 0]]
for k in 1..2:
    for v in 0..3:
        up[k][v] = up[k - 1][up[k - 1][v]]
print(up[1][2])
