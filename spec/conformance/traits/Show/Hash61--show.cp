// expect: ok, output "ok"
use std.strings.{RollingHash61}
let h = RollingHash61("xyxy")
let same = "{h.get(0..2)}" == "{h.get(2..4)}"
print(if same and "{h.get(0..1)}".starts_with("hash "): "ok" else: "bad")
