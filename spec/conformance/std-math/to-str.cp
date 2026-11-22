// expect: ok, output "ff -1010 0 -9223372036854775808"
print(255.to_str(base: 16), (-10).to_str(base: 2), 0.to_str(), (-9_223_372_036_854_775_807 - 1).to_str())
