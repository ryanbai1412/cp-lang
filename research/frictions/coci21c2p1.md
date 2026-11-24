# Friction report: coci21c2p1

Rethought. The v0.2 port worked out the depth from the command's length (`1 + (kind.len - 7) / 3`), kept the counters in an untyped `[0, 0, 0, 0]` list with a dead slot 0, and wrote one `print` per depth. The v0.3 port matches the command to its depth, keeps a `[1..=3]Int` counter, and prints the number with `".".join(..)` over the depths it has. n is typed.

No language gap.
