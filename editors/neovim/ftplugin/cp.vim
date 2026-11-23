if exists('b:did_ftplugin')
    finish
endif
let b:did_ftplugin = 1

setlocal commentstring=//\ %s comments=:///,://
setlocal expandtab shiftwidth=4 softtabstop=4
" `gq` formats through `cpl fmt`, in the style of the nearest cp.toml.
let &l:formatprg = get(g:, 'cpl_path', 'cpl') . ' fmt --stdin'
" `:make` lists type errors and lints through `cpl lint` (compiler/cpl.vim).
compiler cpl
let b:undo_ftplugin = 'setlocal commentstring< comments< expandtab< shiftwidth< softtabstop< formatprg< makeprg< errorformat<'
