-- CP Lang for Neovim 0.10+: the filetype, tree-sitter highlighting when the parser is
-- built, and `cpl lsp` on every .cp buffer with diagnostics, completion, `gd` to the
-- definition and formatting on save. `cpl setup neovim` installs this folder.
-- Set vim.g.cpl_path when cpl is not on the PATH; vim.g.cpl_format_on_save = false
-- and vim.g.cpl_autocomplete = false turn those off.
vim.filetype.add({ extension = { cp = 'cp' } })

-- The server's Run, Trace and Stress lenses name `cpl` arguments; run them in a terminal split.
vim.lsp.commands['cpl.exec'] = function(command)
    vim.cmd('botright new')
    vim.fn.termopen(vim.list_extend({ vim.g.cpl_path or 'cpl' }, command.arguments or {}))
end

-- `:Cpl test`, `:Cpl stress`, `:Cpl submit`...: `cpl <command> <this file> [args]` in a terminal
-- split; bare `:Cpl` runs `test`. Tab completes the command names `cpl --help --json` lists.
local commands
vim.api.nvim_create_user_command('Cpl', function(opts)
    local args = #opts.fargs > 0 and vim.deepcopy(opts.fargs) or { 'test' }
    local file = vim.api.nvim_buf_get_name(0)
    if vim.bo.filetype == 'cp' and file ~= '' then table.insert(args, 2, file) end
    vim.cmd('botright new')
    vim.fn.termopen(vim.list_extend({ vim.g.cpl_path or 'cpl' }, args))
end, {
    nargs = '*',
    desc = 'run cpl on this file',
    complete = function(lead, line)
        if line:match('^%S+%s+%S*$') == nil then return {} end
        if commands == nil then
            local ok, help = pcall(vim.json.decode, vim.fn.system({ vim.g.cpl_path or 'cpl', '--help', '--json' }))
            commands = ok and vim.tbl_map(function(c) return c.name end, help.commands or {}) or {}
        end
        return vim.tbl_filter(function(name) return vim.startswith(name, lead) end, commands)
    end,
})

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
        -- `cpl fmt` on write, in the style of the nearest cp.toml.
        -- Set vim.g.cpl_format_on_save = false to turn it off.
        vim.api.nvim_create_autocmd('BufWritePre', {
            buffer = args.buf,
            group = vim.api.nvim_create_augroup('cplang_fmt_' .. args.buf, {}),
            callback = function()
                if vim.g.cpl_format_on_save == false then return end
                if #vim.lsp.get_clients({ bufnr = args.buf, name = 'cpl' }) == 0 then return end
                vim.lsp.buf.format({ bufnr = args.buf, name = 'cpl', timeout_ms = 2000 })
            end,
        })
        vim.api.nvim_create_autocmd({ 'BufEnter', 'BufWritePost', 'InsertLeave' }, {
            buffer = args.buf,
            group = vim.api.nvim_create_augroup('cplang_lens_' .. args.buf, {}),
            callback = function() vim.lsp.codelens.refresh({ bufnr = args.buf }) end,
        })
    end,
})

-- Once `cpl lsp` attaches: the completion menu as you type `.` (Neovim 0.11+; Ctrl-X Ctrl-O
-- any time) and `gd` unless you mapped it. Formatting on write is set up above.
vim.api.nvim_create_autocmd('LspAttach', {
    group = vim.api.nvim_create_augroup('cplang_attach', {}),
    callback = function(args)
        local client = vim.lsp.get_client_by_id(args.data.client_id)
        if client == nil or client.name ~= 'cpl' then return end
        local buf = args.buf
        if vim.lsp.completion and vim.g.cpl_autocomplete ~= false then
            vim.lsp.completion.enable(true, client.id, buf, { autotrigger = true })
            -- Nothing is inserted until you pick it, unless you set your own completeopt.
            if vim.o.completeopt == 'menu,popup' then pcall(function() vim.bo[buf].completeopt = 'menuone,noselect,popup' end) end
        end
        if vim.fn.maparg('gd', 'n') == '' then
            vim.keymap.set('n', 'gd', vim.lsp.buf.definition, { buffer = buf, desc = 'cpl: go to definition' })
        end
    end,
})
