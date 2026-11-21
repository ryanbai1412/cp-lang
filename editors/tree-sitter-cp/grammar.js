// A token-level grammar for CP Lang: enough structure to highlight, nothing the
// highlighter does not use. Blocks, folding and everything semantic come from
// `cpl lsp`, so a half-typed line never turns the rest of the file into an error.
module.exports = grammar({
  name: 'cp',

  extras: $ => [/\s/],

  word: $ => $.identifier,

  conflicts: $ => [],

  rules: {
    source_file: $ => repeat($._token),

    _token: $ => choice(
      $.comment,
      $.directive,
      $.decorator,
      $.string,
      $.char,
      $.label,
      $.number,
      $.boolean,
      $.none,
      $.self,
      $.keyword,
      $.function_definition,
      $.call,
      $.member,
      $.identifier,
      $.type_identifier,
      $.operator,
      $.punctuation,
    ),

    comment: _ => token(seq('//', /.*/)),

    directive: _ => token(seq('@!', /[A-Za-z_][A-Za-z0-9_]*/)),
    decorator: _ => token(seq('@', /[A-Za-z_][A-Za-z0-9_]*/)),

    // `{...}` interpolations may hold strings of their own: `"{names["x"]}"`.
    string: _ => token(seq(
      optional('r'),
      '"',
      repeat(choice(/[^"\\{\n]/, /\\./, /\{\{/, /\{[^}\n]*\}/)),
      '"',
    )),

    char: _ => token(seq("'", choice(/[^'\\\n]/, /\\./), "'")),
    label: _ => token(seq("'", /[A-Za-z_][A-Za-z0-9_]*/)),

    number: _ => token(choice(
      /0[xX][0-9a-fA-F_]+/,
      /0[bB][01_]+/,
      /[0-9][0-9_]*(\.[0-9][0-9_]*)?([eE][+-]?[0-9][0-9_]*)?/,
    )),

    boolean: _ => choice('true', 'false'),
    none: _ => 'none',
    self: _ => 'self',

    keyword: _ => choice(
      'let', 'var', 'fn', 'struct', 'enum', 'domain', 'type', 'const', 'trait',
      'impl', 'use', 'pub', 'if', 'elif', 'else', 'for', 'while', 'in', 'match',
      'return', 'break', 'continue', 'and', 'or', 'not', 'assert', 'mut',
      'lends', 'yield', 'unchecked',
    ),

    function_definition: $ => prec(2, seq(alias('fn', $.keyword), field('name', $.identifier))),

    call: $ => prec(1, seq(field('function', $.identifier), alias('(', $.punctuation))),

    member: $ => prec.right(1, seq(
      alias('.', $.punctuation),
      field('name', $.identifier),
      optional(field('call', alias('(', $.punctuation))),
    )),

    identifier: _ => /[a-z_][A-Za-z0-9_]*/,
    type_identifier: _ => /[A-Z][A-Za-z0-9_]*/,

    operator: _ => token(choice('..=', '..', /[-+*\/%^&|<>=!?~]+/)),

    punctuation: _ => choice('(', ')', '[', ']', '{', '}', ',', ':', '::', ';', '.'),
  },
});
