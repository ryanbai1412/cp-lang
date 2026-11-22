var up = [[0, 0, 1], [0, 0, 0]]
for v in 0..3:
    up[1][v] = up[0][v] + 1
print(up[1][2])
