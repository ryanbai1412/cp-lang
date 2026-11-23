// Synthesis #2 rule 7: writing through `x!` when x is none is the same runtime error as reading it.
var o: ?Int = None
o! += 2
print(o!)
