// expect: ok, output "-12 -1 -1 255"
print("-12".parse_int() ?? -1, "".parse_int() ?? -1, "+".parse_int() ?? -1, "FF".parse_int(base: 16) ?? -1)
