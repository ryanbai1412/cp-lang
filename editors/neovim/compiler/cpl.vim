" `:make` runs `cpl check` on the buffer and fills the quickfix list.
if exists('current_compiler')
    finish
endif
let current_compiler = 'cpl'
if exists(':CompilerSet') != 2
    command -nargs=* CompilerSet setlocal <args>
endif
execute 'CompilerSet makeprg=' . escape(get(g:, 'cpl_path', 'cpl') . ' check %', ' \|"')
CompilerSet errorformat=%Eerror[%.%#]:\ %m,%Wwarning[%.%#]:\ %m,%Z\ \ -->\ %f:%l:%c,%-G%.%#
