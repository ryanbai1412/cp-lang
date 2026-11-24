// A token-level grammar for CP Lang: enough structure to highlight, nothing the
// highlighter does not use. Blocks, folding and everything semantic come from
// `cpl lsp`, so a half-typed line never turns the rest of the file into an error.
// The words are crates/cpl-syntax's tables, written to editors/words.json by `just gen`.
const words = require('../words.json');
const literals = words.literals;
const oneOf = ws => (ws.length === 1 ? ws[0] : choice(...ws));
const reserved = words.keywords.filter(w => !literals.includes(w) && w !== 'self');

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

    boolean: _ => oneOf(literals.filter(w => /^[a-z]/.test(w))),
    none: _ => oneOf(literals.filter(w => /^[A-Z]/.test(w))),
    self: _ => 'self',

    keyword: _ => oneOf(reserved),

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
