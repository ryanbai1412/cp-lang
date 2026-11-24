// expect: E0302
// A bare value is not the missing-key default: `Counter(0)` asks for a sequence to count, and the default is `init:` by name.
var c: Counter<Int, Int> = Counter(0) // <- here
