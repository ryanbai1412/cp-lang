// `pop()` of a `List<Bool>`, whose C++ keeps a `char` per flag.
var flags: List<Bool> = [true, false, true]
flags.pop()
print(flags.len, [yes(x) for x in flags])
let last: ?Bool = flags.pop()
print(if let b = last: yes(b) else: "none", flags.len)
