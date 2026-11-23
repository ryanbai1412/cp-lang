" CP Lang for Vim: `cpl lsp` through vim-lsp or yegappan/lsp, whichever is
" installed; the highlighting, filetype and indent come from this folder.
" Neovim uses plugin/cplang.lua instead.
if has('nvim') || exists('g:loaded_cplang')
    finish
endif
let g:loaded_cplang = 1

" Without a vimrc that turns them on, .cp files would get no filetype, so no
" ftplugin, highlighting or LSP.
if !exists('g:did_load_filetypes') || !exists('g:did_load_ftplugin') || !exists('g:did_indent_on')
    filetype plugin indent on
endif
if !exists('g:syntax_on')
    syntax on
endif

" vim-lsp leaves completion, hover and go to definition unmapped.
function! s:LspBuffer() abort
    if &filetype !=# 'cp'
        return
    endif
    setlocal omnifunc=lsp#complete tagfunc=lsp#tagfunc
    nmap <buffer> gd <plug>(lsp-definition)
    nmap <buffer> K <plug>(lsp-hover)
endfunction

augroup cplang
    autocmd!
    autocmd User lsp_buffer_enabled call s:LspBuffer()
    autocmd User lsp_setup call lsp#register_server({
                \ 'name': 'cpl',
                \ 'cmd': [cplang#path(), 'lsp'],
                \ 'allowlist': ['cp']})
    autocmd VimEnter * if exists('*LspAddServer')
                \ | call LspAddServer([{'name': 'cpl', 'filetype': ['cp'],
                \     'path': cplang#path(), 'args': ['lsp']}])
                \ | endif
augroup END
