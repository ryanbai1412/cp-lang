// expect: ok
// rule: 12.14.4
// tool: build -o -
// file: cp.toml "[libs]\nbits = \"bits\"\n"
// file: bits/lib.toml "name = \"bits\"\n"
// file: bits/word.cp "pub fn ones(x: Int) -> Int = @intrinsic(\"popcount\")\n"
// has: "cpl::put_int((ll)__builtin_popcountll((uint64_t)(7)))"
// Any library may write `@intrinsic`: a library fn on the table emits the same C++ as std's.
use bits.word.{ones}
print(ones(7))
