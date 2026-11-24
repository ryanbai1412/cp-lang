let b:did_ftplugin = 1

setlocal commentstring=//\ %s comments=:///,://
setlocal expandtab shiftwidth=4 softtabstop=4
" `gq` formats through `cpl fmt`, in the style of the nearest cp.toml.
let &l:formatprg = get(g:, 'cpl_path', 'cpl') . ' fmt --stdin'
" `:make` runs `cpl lint` on the buffer (type errors and lints together) and
" fills the quickfix list.
let &l:makeprg = get(g:, 'cpl_path', 'cpl') . ' lint %'
setlocal errorformat=%Eerror[%.%#]:\ %m,%Wwarning[%.%#]:\ %m,%Z\ \ -->\ %f:%l:%c,%-G%.%#
let b:undo_ftplugin = 'setlocal commentstring< comments< expandtab< shiftwidth< softtabstop< formatprg< makeprg< errorformat<'
