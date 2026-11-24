// here: 3
// C++'s `long long s = 0;` (self-run mistakes): a bare "expected the end of the statement, found a name".
long long s = 0;
for x in [1, 2]:
    s += x
print(s)
