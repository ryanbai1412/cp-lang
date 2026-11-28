// expect: ok, output "1\n2\n2\n3\n3\n1"
// engines: next
for w in [1, 2, 3].ring.windows(2):
    for x in w: print(x)
