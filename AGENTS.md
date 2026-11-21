# AGENTS.md

Start with `plan/OPERATING.md`: the mission, goals, principles, settled
decisions and the working loop. Then `plan/agent-context.md` for the history
behind them and `decisions.md` for every design call and its reason.

The short version:

- Make competitive programming feel effortless: quick to write and revise,
  common mistakes hard to write, failures easy to explain, C++ speed.
- Rethink each solution in the language; never translate C++ line by line.
- Fix every friction with a general mechanism that keeps the language
  coherent. A helper with one use means the general mechanism is missing.
- Small core, no privileged stdlib, one spelling per idea.
- `scripts/check.sh --fast` passes before every push; interpreter and
  generated C++ agree, including on input they reject.
- The history will be public: no internal hostnames, package mirrors,
  session links, secret names or credentials in the repository.
