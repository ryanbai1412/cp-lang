# Intrinsics: one table, no std names in the compiler

Design note for decision 573 and spec R12.14.1-R12.14.5 (W14).

## Picture

```
std/math.cp   pub fn popcount<T: Bits>(x: T) -> Int = @intrinsic("popcount")
                               |
cpl-check     body names an entry of cpl_rt::TABLE; signature checked against entry.sig
                               |
cpl-ir        every call to a fn whose body is @intrinsic(id)  ->  ExprKind::Intrinsic(Prim, args)
              (the fn itself lowers to `return Intrinsic(Prim, params)`, so it is also a value)
                     /                                   \
cpl-interp    TABLE[p].vm (scalar fn over RtVal)     cpl-codegen   TABLE[p].cpp ("__builtin_popcountll((uint64_t)($0))")
              or a fast VM op keyed by Prim                      emitted inline, each $i used once
```

## Rules

- `@intrinsic("id")` is a body form like `@cpp("...")`, usable by any library
  (Small core, no privileged stdlib): a user module copied from std emits the
  same C++ (no-privilege test).
- `cpl-prim` holds `PrimOp` (one Rust enum; `cpl-rt` re-exports it) and `TABLE: [Entry]`: stable id, type
  signature, VM implementation, C++ snippet, the runtime helper it needs.
  `table_count` test pins the length; growing it needs a decision entry.
- The compiler matches `PrimOp` values, never std names. The name-ban gate
  (debt-ratchet `std_names_outside_rt`) counts public std names used as string
  literals in compiler source outside `cpl-rt`; it only shrinks.
- In-place storage primitives (`sort_by_key`, `sort_by_less`) take their `mut`
  argument as a place; their VM side is `Vm::Host` (the interpreter owns list
  storage), their C++ is `std::stable_sort` over the contiguous span with the
  closure inlined: no tagged copies.

## Why not

- A std-only `@builtin` marker: privileges std. Rejected by the principle.
- `@cpp` plus a VM twin per site: the VM twin has to be Rust, so it lives in a
  table anyway; naming the entry is the smaller surface and keeps C++ and VM in
  one file.
- Keeping `Builtin` and adding std wrappers: a wrapper call per use (perf rule:
  same C++ builtin, no overhead) and the name arms stay.

## Migration order

table + popcount; abs/floor/ceil/round/trunc/sqrt; bits; float fns;
`sort_by_key`/`sort_by_less` (Seq writes the std body); HashMap sites.
Each batch deletes its compiler arms and reports `ir_builtin_names`,
`backend_std_names`, `prelude_privilege`.
