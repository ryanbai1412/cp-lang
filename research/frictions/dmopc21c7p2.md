# Friction report: dmopc21c7p2

No gap. The scarf is a doubly linked list over `Spot = 0..=N`, with spot 0 as the front and `next: [Spot]?Spot`, so the old tail sentinel and the scan of every operation for the largest colour are gone. Operations read straight into `Spot`, and the walk ends at `None`.

## Measurements

gen.py --max (N = 1e6, Q = 5e5 one-colour moves): port 0.11 s, accepted C++ 0.34 s.
