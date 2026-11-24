// here: 3
// Python's `Counter(0)` habit (stdlib 78a8a7aee made `init:` named-only): the error had no fix.
var c = Counter(0)
c[2] += 1
print(c[2])
