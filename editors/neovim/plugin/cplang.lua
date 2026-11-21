-- CP Lang for Neovim 0.10+: the filetype, tree-sitter highlighting when the parser is
-- built, and `cpl lsp` on every .cp buffer.
-- Set vim.g.cpl_path when cpl is not on the PATH.
vim.filetype.add({ extension = { cp = 'cp' } })

-- The server's Run, Trace and Stress lenses name `cpl` arguments; run them in a terminal split.
vim.lsp.commands['cpl.exec'] = function(command)
    vim.cmd('botright split')
    vim.fn.termopen(vim.list_extend({ vim.g.cpl_path or 'cpl' }, command.arguments or {}))
end

vim.api.nvim_create_autocmd('FileType', {
    pattern = 'cp',
    group = vim.api.nvim_create_augroup('cplang', {}),
    callback = function(args)
        -- Tree-sitter highlighting once parser/cp.so is on the runtime path (see
        -- editors/README.md); the Vim syntax otherwise.
        pcall(vim.treesitter.start, args.buf, 'cp')
        local file = vim.api.nvim_buf_get_name(args.buf)
        vim.lsp.start({
            name = 'cpl',
            cmd = { vim.g.cpl_path or 'cpl', 'lsp' },
            init_options = { codeLens = true },
            root_dir = vim.fs.root(args.buf, { '.git' }) or vim.fs.dirname(file),
        }, { bufnr = args.buf })
        vim.api.nvim_create_autocmd({ 'BufEnter', 'BufWritePost', 'InsertLeave' }, {
            buffer = args.buf,
            group = vim.api.nvim_create_augroup('cplang_lens_' .. args.buf, {}),
            callback = function() vim.lsp.codelens.refresh({ bufnr = args.buf }) end,
        })
    end,
})
