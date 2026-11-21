; Helix's capture names for editors/tree-sitter-cp/queries/highlights.scm;
; editors/smoke.sh checks the patterns match.
(comment) @comment
(string) @string
(char) @constant.character
(label) @label
(number) @constant.numeric
(boolean) @constant.builtin.boolean
(none) @constant.builtin
(self) @variable.builtin
(directive) @keyword.directive
(decorator) @attribute
(keyword) @keyword
(function_definition name: (identifier) @function)
(call function: (identifier) @function)
(member name: (identifier) @function.method call: (_))
(member name: (identifier) @variable.other.member !call)
((type_identifier) @constant
  (#match? @constant "^[A-Z][A-Z0-9_]+$"))
((type_identifier) @type
  (#not-match? @type "^[A-Z][A-Z0-9_]+$"))
(operator) @operator
(punctuation) @punctuation.delimiter
