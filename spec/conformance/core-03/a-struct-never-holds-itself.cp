// expect: E0322
// rule: 03.2.9
// A struct stores its fields inline, so `l: ?N` would make every `N` hold
// another `N`; the help names `std.persistent.PArena`.
struct N:
    k: Int
    l: ?N  // <- here
print(N(1, None).k)
