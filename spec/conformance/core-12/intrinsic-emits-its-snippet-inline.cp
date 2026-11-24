// expect: ok
// rule: 12.14.3
// tool: build -o -
// has: "cpl::put_int((ll)__builtin_popcountll((uint64_t)(7))"
// The C++ of a call of an `@intrinsic` fn is its entry's snippet, with no call of the fn.
print(popcount(7))
