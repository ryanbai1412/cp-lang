# std.io doc notes

Choices made where the spec is silent, and contradictions found, while bringing
`spec/std/io.cp` to the doc standard.

- `read_all` (no `until`): core says "reading after it is an error" without saying when; chose a runtime trap at the later read (reading past the end of input), since the read cannot be rejected statically in general.
- `read_all(until: f)` with no matching value: chose to read to end of input and return everything, like the plain form; the sentinel is a stopping hint, not a required token, and trapping would make the two forms disagree on well-formed input.
- `Wire.new` called twice: chose "allowed, no further effect"; interactive mode is a global flush policy and a second handle cannot make it more interactive.
- `Wire.ask`: chose that the newline after `query` is always printed and the answer is read by the same rules (and traps) as `read<T>()`; §10.4 says "print a line, flush, and read".
- Examples for `Wire.ask` and `read_all` name the stdin they assume in a trailing parenthetical; the test generator must feed it.
- Contradiction: spec/core/10-io.md §10.5 gives `read_all<T: Read>(...)`, the stub has `read_all<T>(...)` without the `Read` bound. Left as is. (resolved: stub kept (no `Read` trait exists); core chapter fixed)
