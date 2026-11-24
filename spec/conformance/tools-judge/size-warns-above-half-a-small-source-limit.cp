// expect: ok
// rule: judge.4
// tool: build --judge tiny -o /dev/null
// file: cp.toml "[judge.tiny]\nextends = \"generic\"\nsource_kb = 4\n"
// has: "W0201"
// A judge whose limit is under 64 KB scales the warning with it: above half its `source_kb`, here 2 KB, the build warns (decision 188).
print(1)
