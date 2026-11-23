// expect: E0504 at 2
// Several domains on one line is planned syntax; the error names the one-per-line form.
domain A = 0..3, B = 0..4
print(1)
