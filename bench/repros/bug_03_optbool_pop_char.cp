// Repro: `cpl::OptBool(cpl::pop(list_of_bool))` fails g++ compilation.
// Discovered in: proto/ports/scout_batch_5/btoi19p6.cp (g++ compile failure)
// Cause: `List<Bool>` lowers to `std::vector<char>` (to avoid vector<bool> proxy),
// so `cpl::pop(xs)` returns `std::optional<char>`, which `cpl::OptBool` rejects
// because `OptBool` only accepts `std::optional<bool>`.

let n: Int = read()
var flags: List<Bool> = []
for i in 0..n:
    flags.push(i % 2 == 0)
let last: ?Bool = flags.pop()
if let b = last:
    print(if b: 1 else: 0)
