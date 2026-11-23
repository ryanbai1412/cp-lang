// expect: E0302 at 2
// rule: 04.4.22
// Conversions apply to arguments, not receivers: a `List<Char>` does not call a `self: Str` fn, or `['R', 'W'].join("")` would join `""` with `"RW"` as the separator and print an empty line.
print(['R', 'W'].join(""))
