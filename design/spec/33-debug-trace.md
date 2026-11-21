# 33. Debug, trace and tool coverage

**Layer:** tools. Debug builds keep every check and record a trace; nothing here changes program meaning. Status: [proposed].

## 33.1 Tool coverage

Every construct here must work in: the debug interpreter (positions shown as labels, one-based on request),
the HTML tracer (arrays shown with their index space, cuts drawn between cells), `cp explain` (widths,
hidden arguments, remaining checks), stress generators (`gen<Graph<V>>`, `gen(1..=8)` for sizes), the
formatter, and the LSP (hover shows the index space of every subscript).
