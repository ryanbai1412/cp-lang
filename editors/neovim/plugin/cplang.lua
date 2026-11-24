-- CP Lang for Neovim 0.12 or later: the filetype, tree-sitter highlighting when the parser is
-- built, and `cpl lsp` on every .cp buffer, which does the rest: diagnostics, completion,
-- the lenses, `:Cpl` runs and what they show, and formatting on save (willSaveWaitUntil).
-- `cpl setup neovim` installs this folder and sets vim.g.cpl_path, else `cpl` on the PATH runs;
-- vim.g.cpl_format_on_save = false and vim.g.cpl_autocomplete = false turn those off.
vim.filetype.add({ extension = { cp = 'cp' } })

local function client() return vim.lsp.get_clients({ bufnr = 0, name = 'cpl' })[1] end

-- The server shows each run's panel as a message, its status, and its first wrong call
-- as a diagnostic on that line.
vim.lsp.config('cpl', {
    cmd = { vim.F.if_nil(vim.g.cpl_path, 'cpl'), 'lsp' },
    filetypes = { 'cp' },
    root_markers = { '.git' },
    init_options = { codeLens = true, panel = 'message', formatOnSave = vim.g.cpl_format_on_save ~= false },
    -- The completion menu as you type `.` (Ctrl-X Ctrl-O any time), and `gd` to the definition.
    on_attach = function(c, buf)
        vim.lsp.completion.enable(vim.g.cpl_autocomplete ~= false, c.id, buf, { autotrigger = true })
        vim.keymap.set('n', 'gd', vim.lsp.buf.definition, { buffer = buf, desc = 'cpl: go to definition' })
    end,
})
vim.lsp.enable('cpl')
vim.lsp.codelens.enable(true)

vim.api.nvim_create_autocmd('FileType', {
    pattern = 'cp',
    group = vim.api.nvim_create_augroup('cplang', {}),
    callback = function(args) pcall(vim.treesitter.start, args.buf, 'cp') end,
})

-- `:Cpl <command>` runs `cpl <command>` on this file through the server; bare `:Cpl` is `cpl test`.
vim.api.nvim_create_user_command('Cpl', function(opts)
    client():exec_cmd({ title = 'cpl', command = 'cpl.command', arguments = { vim.api.nvim_buf_get_name(0), unpack(opts.fargs) } })
end, {
    nargs = '*',
    desc = 'run cpl on this file',
    complete = function(_, line) return client():request_sync('cpl/commands', { line = line }, 2000, 0).result end,
})

-- Time travel through `cpl dap` when nvim-dap is installed: `:DapContinue` on a `.cp` file
-- records the run and stops at its first step; step back with `require('dap').step_back()`.
-- An adapter or configurations of your own for `cp` stay.
vim.api.nvim_create_autocmd('FileType', {
    pattern = 'cp',
    group = vim.api.nvim_create_augroup('cplang_dap', {}),
    callback = function()
        pcall(function()
            local dap = require('dap')
            dap.adapters.cp = vim.F.if_nil(dap.adapters.cp, { type = 'executable', command = vim.F.if_nil(vim.g.cpl_path, 'cpl'), args = { 'dap' } })
            dap.configurations.cp = vim.F.if_nil(dap.configurations.cp, {
                { type = 'cp', request = 'launch', name = 'Debug this file', program = '${file}', stopOnEntry = true },
                {
                    type = 'cp', request = 'launch', name = 'Debug on an input file', program = '${file}',
                    input = function() return vim.fn.input('Input file: ', vim.fn.expand('%:p:h') .. '/', 'file') end,
                },
            })
        end)
    end,
})
