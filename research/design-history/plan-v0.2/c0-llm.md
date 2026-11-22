# 12. LLM-friendliness

Status: proposed. A new language starts at a disadvantage: models haven't
seen it. Agents will also do much of the porting and stdlib work. So the
language and tools should be easy to learn from a small context and easy to
drive from a program.

## 12.1 A compact spec: `llms.txt`

A single file under 30k tokens, generated from the plan and the stdlib docs
so it can't drift:

- the syntax in one page, with every construct shown once;
- the core rules: domains and bounds, optionals, ownership (variables own,
  parameters borrow, `mut` at both ends), traits vs operation values,
  program structure;
- the 50 most-used stdlib APIs with signatures and one-line examples;
- the 20 most common errors with their fixes;
- 20 complete programs from the cookbook, each with the C++ it replaces.

## 12.2 Tools an agent can drive

- `cp check --json`: diagnostics as JSON, each with its code, span,
  message, and a machine-applicable fix where one exists.
- `cp fix`: applies those fixes; `--dry-run` prints the diff.
- `cp explain --json`: types, ranges, sizes and costs for a file.
- `cp port`: a C++ draft with `// PORT:` marks (doc 10).
- `cp test --json` and `cp stress --json`: results, the smallest failing
  input, and the first differing output line.

The intended loop is write, check, read structured errors, fix, test, and
every step has a stable machine format.

## 12.3 Messages that work for models too

The same properties that help a human help a model: one error per root
cause, a concrete "write this instead", and no reliance on reading the
generated C++. Messages never say only "type mismatch"; they say which
value, which expected type, and why it was expected.

## 12.4 Evaluation

About 100 problems solved by a model given only `llms.txt` and the tools,
scored on compile rate, correctness and number of fix iterations. Tracked in
CI like the corpus. A drop means the spec or the messages got worse.
