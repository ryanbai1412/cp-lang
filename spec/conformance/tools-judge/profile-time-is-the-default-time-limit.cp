// expect: exit 1
// rule: judge.6
// tool: test {case}
// file: cp.toml "[judge.tiny]\nextends = \"generic\"\ntime_ms = 1\n"
// file: profile-time-is-the-default-time-limit.tests/1.in "\n"
// has: "TLE"
// has: "0/1 passed"
// judge: tiny
// A problem with no `[limits]` gets its judge profile's `time_ms`: three million steps fit the generic judge's 2 s but not this judge's 1 ms, so `cpl test` reports TLE.
var s = 0
for i in 0..3000000:
    s += i % 7
print(s)
