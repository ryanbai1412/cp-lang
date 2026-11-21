; Neovim's capture names. No two patterns capture the same node, so the result
; does not depend on which match an editor lets win; editors/helix and
; editors/zed carry the same patterns under their own names.
(comment) @comment
(string) @string
(char) @character
(label) @label
(number) @number
(boolean) @boolean
(none) @constant.builtin
(self) @variable.builtin
(directive) @keyword.directive
(decorator) @attribute
(keyword) @keyword
(function_definition name: (identifier) @function)
(call function: (identifier) @function.call)
(member name: (identifier) @function.method.call call: (_))
(member name: (identifier) @variable.member !call)
((type_identifier) @constant
  (#match? @constant "^[A-Z][A-Z0-9_]+$"))
((type_identifier) @type
  (#not-match? @type "^[A-Z][A-Z0-9_]+$"))
(operator) @operator
(punctuation) @punctuation.delimiter
