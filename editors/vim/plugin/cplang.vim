" CP Lang for Vim: `cpl lsp` through vim-lsp or yegappan/lsp, whichever is
" installed; the highlighting, filetype and indent come from editors/neovim.
" Neovim uses editors/neovim/plugin/cplang.lua instead.
let g:loaded_cplang = 1

" Without a vimrc that turns them on, .cp files would get no filetype, so no
" ftplugin, highlighting or LSP.
filetype plugin indent on
syntax enable

augroup cplang
    autocmd!
    autocmd User lsp_setup call lsp#register_server({
                \ 'name': 'cpl',
                \ 'cmd': [get(g:, 'cpl_path', 'cpl'), 'lsp'],
                \ 'allowlist': ['cp']})
    autocmd User LspSetup call LspAddServer([{'name': 'cpl', 'filetype': ['cp'],
                \ 'path': get(g:, 'cpl_path', 'cpl'), 'args': ['lsp']}])
augroup END
