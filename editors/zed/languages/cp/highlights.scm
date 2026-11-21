; Zed's capture names for editors/tree-sitter-cp/queries/highlights.scm;
; editors/smoke.sh checks the patterns match.
(comment) @comment
(string) @string
(char) @string.special
(label) @label
(number) @number
(boolean) @boolean
(none) @constant
(self) @variable.special
(directive) @preproc
(decorator) @attribute
(keyword) @keyword
(function_definition name: (identifier) @function)
(call function: (identifier) @function)
(member name: (identifier) @function call: (_))
(member name: (identifier) @property !call)
((type_identifier) @constant
  (#match? @constant "^[A-Z][A-Z0-9_]+$"))
((type_identifier) @type
  (#not-match? @type "^[A-Z][A-Z0-9_]+$"))
(operator) @operator
(punctuation) @punctuation.delimiter
