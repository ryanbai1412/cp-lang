# oly20practice139

Score: pending
Dream: pending

Follow-up: std `MonotoneHull(max)` now compares in I128 (97051b8e4), so the local `Upper`
hull is gone; each block is a `MonotoneHull<()>` queried with `at_next`. Same output, same
native time at --max (1.43 s against 1.41 s for the old port and 1.45 s for the AC).
