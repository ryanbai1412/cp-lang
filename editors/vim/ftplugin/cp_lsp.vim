" A .cp buffer in Vim: vim-lsp leaves completion, hover and go to definition
" unmapped, so ^X^O, K and gd go to `cpl lsp` here.
setlocal omnifunc=lsp#complete tagfunc=lsp#tagfunc
nmap <buffer> gd <plug>(lsp-definition)
nmap <buffer> K <plug>(lsp-hover)
let b:undo_ftplugin .= '| setlocal omnifunc< tagfunc< | silent! nunmap <buffer> gd | silent! nunmap <buffer> K'
