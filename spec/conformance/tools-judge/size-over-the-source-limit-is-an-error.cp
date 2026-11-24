// expect: exit 1
// rule: judge.4
// tool: build --judge tiny -o /dev/null
// file: cp.toml "[judge.tiny]\nextends = \"generic\"\nsource_kb = 1\n"
// has: "over the 1024-byte limit of judge `tiny`"
// has: "size of the C++ by part"
// C++ above the judge's `source_kb` is an error, not a warning, and the build breaks the file down by part.
print(1)
