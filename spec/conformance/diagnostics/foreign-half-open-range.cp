// expect: E0107 at 2
// `..<` is a foreign spelling of `..`, which is already half-open.
for i in 0..<3:
    print(i)
