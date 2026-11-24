// here: 4
// C++'s `a.erase(unique(..), a.end())` (self-run mistakes): the fix renamed `erase` and left two errors.
var a = [1, 1, 2, 2, 3]
a.erase(unique(a.begin(), a.end()), a.end());
print(a)
