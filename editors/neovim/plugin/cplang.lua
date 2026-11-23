-- CP Lang for Neovim 0.10+: the filetype, tree-sitter highlighting when the parser is
-- built, and `cpl lsp` on every .cp buffer with diagnostics, completion, `gd` to the
-- definition and formatting on save. `cpl setup neovim` installs this folder.
-- cpl is found as autoload/cplang.vim says; vim.g.cpl_path overrides it; vim.g.cpl_format_on_save = false
-- and vim.g.cpl_autocomplete = false turn those off.
vim.filetype.add({ extension = { cp = 'cp' } })

-- The solution's line of the first wrong call `cpl stress` or `cpl locate` printed
-- (`solution tri(k: 1) = 0 at line 4` under `first wrong call: ...`), and that headline.
local function divergence(lines)
    local headline
    for _, line in ipairs(lines) do
        local h = line:match('^%s*(first wrong call: .-)%s*$') or line:match('^%s*(first differing value: .-)%s*$')
        if h then headline = h end
        local at = headline and line:match('^%s+solution%s.-%sat line (%d+)%s*$')
        if at then return { line = tonumber(at), message = headline } end
    end
end

-- `cpl <args>` in a terminal split. When it names the first wrong call, that line of the
-- file shows it as a warning until the next run.
local wrong_call = vim.api.nvim_create_namespace('cpl_divergence')
local function run_in_terminal(args)
    local source = vim.api.nvim_get_current_buf()
    vim.diagnostic.reset(wrong_call, source)
    vim.cmd('botright new')
    local term = vim.api.nvim_get_current_buf()
    vim.fn.termopen(vim.list_extend({ vim.fn['cplang#path']() }, args), {
        on_exit = function()
            -- The last of the output can reach the buffer just after the exit (Windows' ConPTY).
            vim.defer_fn(function()
                if not (vim.api.nvim_buf_is_valid(term) and vim.api.nvim_buf_is_valid(source)) then return end
                local found = divergence(vim.api.nvim_buf_get_lines(term, 0, -1, false))
                if found == nil then return end
                vim.diagnostic.set(wrong_call, source, { {
                    lnum = found.line - 1, col = 0, source = 'cpl',
                    severity = vim.diagnostic.severity.WARN, message = found.message,
                } })
            end, 200)
        end,
    })
end

-- The server's Run, Trace and Stress lenses name `cpl` arguments; run them in a terminal split.
vim.lsp.commands['cpl.exec'] = function(command)
    run_in_terminal(command.arguments or {})
end

-- `:Cpl test`, `:Cpl stress`, `:Cpl submit`...: `cpl <command> <this file> [args]` in a terminal
-- split; bare `:Cpl` runs `test`. Tab completes the command names `cpl --help --json` lists.
local commands
vim.api.nvim_create_user_command('Cpl', function(opts)
    local args = #opts.fargs > 0 and vim.deepcopy(opts.fargs) or { 'test' }
    local file = vim.api.nvim_buf_get_name(0)
    if vim.bo.filetype == 'cp' and file ~= '' then table.insert(args, 2, file) end
    run_in_terminal(args)
end, {
    nargs = '*',
    desc = 'run cpl on this file',
    complete = function(lead, line)
        if line:match('^%S+%s+%S*$') == nil then return {} end
        if commands == nil then
            local ok, help = pcall(vim.json.decode, vim.fn.system({ vim.fn['cplang#path'](), '--help', '--json' }))
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
            cmd = { vim.fn['cplang#path'](), 'lsp' },
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
        if vim.lsp.codelens.enable then
            -- Neovim 0.12+ keeps the lenses current itself.
            vim.lsp.codelens.enable(true, { bufnr = args.buf })
        else
            vim.api.nvim_create_autocmd({ 'BufEnter', 'BufWritePost', 'InsertLeave' }, {
                buffer = args.buf,
                group = vim.api.nvim_create_augroup('cplang_lens_' .. args.buf, {}),
                callback = function() vim.lsp.codelens.refresh({ bufnr = args.buf }) end,
            })
        end
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

-- Time travel through `cpl dap` when nvim-dap is installed: `:DapContinue` on a `.cp` file
-- records the run and stops at its first step; step back with `require('dap').step_back()`.
vim.api.nvim_create_autocmd('FileType', {
    pattern = 'cp',
    group = vim.api.nvim_create_augroup('cplang_dap', {}),
    callback = function()
        local ok, dap = pcall(require, 'dap')
        if not ok or dap.adapters.cp ~= nil then return end
        dap.adapters.cp = { type = 'executable', command = vim.fn['cplang#path'](), args = { 'dap' } }
        dap.configurations.cp = dap.configurations.cp or {
            { type = 'cp', request = 'launch', name = 'Debug this file', program = '${file}', stopOnEntry = true },
            {
                type = 'cp', request = 'launch', name = 'Debug on an input file', program = '${file}',
                input = function() return vim.fn.input('Input file: ', vim.fn.expand('%:p') .. '.tests/', 'file') end,
            },
        }
    end,
})
