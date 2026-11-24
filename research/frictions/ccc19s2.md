# Friction Report: ccc19s2

No gap: the port is the dream with typed bounds (`[t]4..=1e6`) in place of `read [t] Int`.

## Measurements

- Tokens: port 80, original 451
- Max input (`gen.py --max`): port 0.02 s, 11612 KB, original 0.00 s, 6124 KB
- Emitted C++: 11178 bytes
