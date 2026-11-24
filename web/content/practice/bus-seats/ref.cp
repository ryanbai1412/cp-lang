let (n, c): (1..=2e5, 1..=1e9) = read()
var inside = 0
var refused = 0
for _ in 0..n:
    let (off, on): (0..=1e9, 0..=1e9) = read()
    inside -= min(off, inside)
    refused += max(0, inside + on - c)
    inside = min(c, inside + on)
print(inside, refused)
