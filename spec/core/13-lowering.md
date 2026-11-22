# 13. Lowering to C++

Status: draft for 0.3.0. This is the contract between the language and
the generated C++17: what each form becomes, what it costs, and what
`cpl explain` reports. Sources: decisions 31, 52, 148, 149; archived
`design/spec/18`; `03-numbers-layout.md` for widths and niches.

## 13.1 The output file

- One deterministic C++17 file: a stamp, the used runtime pieces, the
  used library instances, then `main`. Same source and pins give the
  same bytes. Warn at 48 KB, fail at 64 KB (profile-adjustable).
- `main` starts a thread with the profile's stack (default 1 GB,
  halving down to 8 MB) and runs the program on it.
- Input is one bulk `fread` and a hand-written parser; output is one
  buffer written at exit or flush.
- Release builds compile with the profile's flags and emit no checks
  (chapter 9). The lowering adds no undefined behaviour to valid
  programs, and avoids it wherever that is free: signed arithmetic that
  may overflow is done in unsigned and cast back, shifts are masked, and
  every array is filled at construction. Unchecked indexes and `x!` on an
  invalid program are as unsafe as the same C++ (§9.2).

## 13.2 Values

| Form | C++ |
|---|---|
| `Int`, ranges | `int64_t` in locals; storage at the range's width (`uint8_t` ... `int64_t`) |
| `U64`, `I128`, `F64`, `Bool`, `Char` | `uint64_t`, `__int128`, `double`, `bool`, `char` |
| position `v: V` | unsigned of `V`'s storage width |
| cut | the same width, holding the count before it |
| `?T` with a niche | `T`'s storage with the niche value as `none` |
| `?T` without one | `T` plus a flag byte, aligned |
| tuple, record, struct | a C++ struct, fields in storage order |
| enum | niche-filled payload, or tag beside the largest payload |
| `[D]T` | pointer plus count, heap buffer; never a stack VLA |
| `List<T>`, `Str` | an owned growable buffer (`std::vector`-like runtime type) |
| `Mod<P>` constant | `uint32_t` or `uint64_t` with Montgomery or Barrett constants folded |
| `Mod(p)` runtime | the same plus constants in the fresh type's static slot |
| fresh-type data | one static slot per declaration site, saved and restored across recursive re-entry |

Large static arrays (over 4 KB) and every runtime-sized array live on
the heap. Locals of plain types live in registers or the stack.

## 13.3 Functions, closures, generators

- Every generic instantiation is a separate C++ function with a mangled
  name; there are no C++ templates in the output for language generics.
- A function value with a known target is passed as an empty struct and
  inlined; captures are passed as references at each call (chapter 6).
  There is no `std::function` and no indirect call.
- A non-recursive generator inlines its body with the loop body in place
  of `yield`. A recursive generator becomes a recursive function taking
  the loop body as a parameter; when the body can `break` or `return`, a
  stop flag is threaded through that specialization only.
- `@memo(array)` is a lazily allocated dense table of `?R` (jagged when
  bounds depend on earlier parameters); `@memo(map)` is an open-
  addressing table keyed by the parameters' storage.
- Views are ordinary structs holding a pointer (lent) or the value
  (owned) plus their closures; walking them is a loop with no
  allocation.

## 13.4 Ownership

Ownership has no runtime representation. Lends are pointers or
references, moves are bitwise and leave the source dead (no destructor
runs for a moved-from value), and `.clone()` is an explicit deep copy.
A struct whose fields may be lent or owned is monomorphized on that
choice only when the field's storage differs (a lent `List` is a
pointer, an owned one is the buffer); `cpl explain` lists such copies.

## 13.5 Library specializations

Each library structure documents its representation and every
law-selected specialization, and codegen picks exactly that one:
iterative bottom-up segment trees in one flat array, no push-down when
tags commute, sliding folds by group/selective/general, sparse tables
for idempotent ops, CSR graphs, `Rooted` arrays in preorder. `cpl
explain` names the choice and the law that allowed it.

## 13.6 What `cpl explain` reports

For any program: every type's size, storage order, padding and niche;
every array's element width and peak bytes at the input bounds; total
peak memory against the profile; each specialization and the law or
bound behind it; each check remaining in checked builds and which are
kept in release; each `@cache`, `@trusted` and `@cpp` site; each
generic instance and context copy with its output bytes. The LSP shows
the same data inline (`tools/lsp.md`).

## 13.7 Later

Profile-driven vectorization: a judge profile names the instruction set
(`isa: avx2`), codegen emits the matching `#pragma GCC target`, narrows
arithmetic where ranges allow and vectorizes provable loop shapes;
`explain` says which loops vectorized and why not.
