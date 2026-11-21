-- CP Lang for Neovim 0.10+: the filetype, and `cpl lsp` on every .cp buffer.
-- Set vim.g.cpl_path when cpl is not on the PATH.
vim.filetype.add({ extension = { cp = 'cp' } })

vim.api.nvim_create_autocmd('FileType', {
    pattern = 'cp',
    group = vim.api.nvim_create_augroup('cplang', {}),
    callback = function(args)
        local file = vim.api.nvim_buf_get_name(args.buf)
        vim.lsp.start({
            name = 'cpl',
            cmd = { vim.g.cpl_path or 'cpl', 'lsp' },
            root_dir = vim.fs.root(args.buf, { '.git' }) or vim.fs.dirname(file),
        }, { bufnr = args.buf })
    end,
})
