" CP Lang for Vim: `cpl lsp` through vim-lsp or yegappan/lsp, whichever is
" installed; the highlighting, filetype and indent come from this folder.
" Neovim uses plugin/cplang.lua instead.
if has('nvim') || exists('g:loaded_cplang')
    finish
endif
let g:loaded_cplang = 1

augroup cplang
    autocmd!
    autocmd User lsp_setup call lsp#register_server({
                \ 'name': 'cpl',
                \ 'cmd': [get(g:, 'cpl_path', 'cpl'), 'lsp'],
                \ 'allowlist': ['cp']})
    autocmd VimEnter * if exists('*LspAddServer')
                \ | call LspAddServer([{'name': 'cpl', 'filetype': ['cp'],
                \     'path': get(g:, 'cpl_path', 'cpl'), 'args': ['lsp']}])
                \ | endif
augroup END
