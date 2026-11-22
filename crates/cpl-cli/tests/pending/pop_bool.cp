// `pop()` of a `List<Bool>`, whose C++ keeps a `char` per flag.
var flags: List<Bool> = [true, false, true]
flags.pop()
print(flags.len, flags)
let last: ?Bool = flags.pop()
print(last, flags.len)
