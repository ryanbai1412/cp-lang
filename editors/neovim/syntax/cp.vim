" CP Lang. The reserved words are `TokenKind::keyword` in crates/cpl-syntax/src/token.rs.
if exists('b:current_syntax')
    finish
endif

syntax keyword cpKeyword let var fn struct enum domain type const trait impl use pub
syntax keyword cpKeyword if elif else for while in match return break continue
syntax keyword cpKeyword and or not assert mut self lends yield unchecked
syntax keyword cpBoolean true false none
syntax match cpType '\<\u\w*\>'
syntax match cpDecorator '@\h\w*'
syntax match cpLabel "'\h\w*\>'\@!"
syntax match cpNumber '\<0[xX][0-9a-fA-F_]\+\>'
syntax match cpNumber '\<0[bB][01_]\+\>'
syntax match cpNumber '\<\d[0-9_]*\%(\.\d[0-9_]*\)\=\%([eE][+-]\=\d[0-9_]*\)\=\>'
syntax match cpChar "'\%(\\.\|[^\\']\)'"
syntax region cpString start='\<r"' end='"\|$' oneline
syntax region cpString start='"' skip='\\.' end='"\|$' oneline contains=cpEscape,cpInterpolation
syntax match cpEscape '\\.\|{{\|}}' contained
syntax region cpInterpolation matchgroup=cpDelimiter start='{\ze[^{]' end='}' contained contains=TOP oneline
syntax keyword cpTodo TODO FIXME XXX contained
syntax match cpComment '//.*$' contains=cpTodo
syntax match cpDocComment '///.*$' contains=cpTodo

highlight default link cpKeyword Keyword
highlight default link cpBoolean Boolean
highlight default link cpType Type
highlight default link cpDecorator PreProc
highlight default link cpLabel Label
highlight default link cpNumber Number
highlight default link cpChar Character
highlight default link cpString String
highlight default link cpEscape SpecialChar
highlight default link cpDelimiter Delimiter
highlight default link cpTodo Todo
highlight default link cpComment Comment
highlight default link cpDocComment SpecialComment

let b:current_syntax = 'cp'
