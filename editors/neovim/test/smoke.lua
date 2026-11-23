-- nvim --headless --clean -l editors/neovim/test/smoke.lua CPL REPO
-- Installs the plugin with `cpl setup neovim`, opens a .cp file with a type error and checks the
-- filetype, the highlighting, the server's diagnostic, its hover and the run lens beside
-- the sample `i1`, run through `cpl.exec`; completion, `gd` and format on write; `:Cpl check` and its completion; then, with the
-- tree-sitter parser built, the highlighting the shared queries give.
local cpl, repo = arg[1], arg[2]
vim.g.cpl_path = cpl

local function check(ok, what)
    if not ok then
        io.stderr:write('neovim smoke: ' .. what .. '\n')
        os.exit(1)
    end
end

-- The documented one step: `cpl setup neovim` puts the plugin in a start package.
-- The real path, so the server and the client name files alike (macOS's /var is
-- /private/var; Windows's temp folder may be a short 8.3 name).
local dir = vim.fn.tempname()
vim.fn.mkdir(dir, 'p')
dir = vim.fs.normalize(vim.uv.fs_realpath(dir) or dir)
local package = dir .. '/site/pack/cpl/start/cplang'
local setup = vim.system({ cpl, 'setup', 'neovim', '--dir', package }):wait()
check(setup.code == 0, 'cpl setup neovim: ' .. (setup.stderr or ''))
-- Startup has already loaded the packages; add this one as `:packloadall`
-- would and source what it would have.
vim.opt.packpath:prepend(dir .. '/site')
vim.opt.runtimepath:prepend(package)
vim.cmd('runtime! plugin/**/*.vim plugin/**/*.lua ftdetect/*.vim')
check(#vim.api.nvim_get_runtime_file('plugin/cplang.lua', true) == 1, 'the package is not on the path')
vim.cmd('filetype plugin on')
vim.cmd('syntax on')

local file = dir .. '/a.cp'
vim.fn.writefile({ '5' }, dir .. '/i1')
--- Whether the terminal buffer `term` shows `want`. Headless Neovim on Windows draws nothing in a
--- terminal buffer (the job runs: CI shows `[Process exited 1]`), so there the check runs
--- `cpl <args>` itself and looks at what it prints.
local function terminal_shows(term, want, args)
    local shown = ''
    -- Windows's pseudo-console can draw the output after the job has exited.
    vim.wait(10000, function()
        shown = table.concat(vim.api.nvim_buf_get_lines(term, 0, -1, false), '\n')
        return shown:find(want, 1, true) ~= nil
    end, 100)
    if shown:find(want, 1, true) or vim.fn.has('win32') == 0 then
        if not shown:find(want, 1, true) then io.stderr:write('terminal: ' .. vim.inspect(shown) .. '\n') end
        return shown:find(want, 1, true) ~= nil
    end
    local direct = vim.system(vim.list_extend({ cpl }, vim.deepcopy(args))):wait()
    local printed = (direct.stdout or '') .. (direct.stderr or '')
    io.stderr:write(('`cpl %s` gives %d: %s\n'):format(table.concat(args, ' '), direct.code, printed))
    return printed:find(want, 1, true) ~= nil
end

vim.fn.writefile({ 'fn main():', '    let n: Int = read()', '    let s: Str = n', '    print(n + 1)' }, file)
vim.cmd.edit(file)
check(vim.bo.filetype == 'cp', 'filetype is ' .. vim.bo.filetype)
check(vim.bo.commentstring == '// %s', 'commentstring is ' .. vim.bo.commentstring)
local group = vim.fn.synIDattr(vim.fn.synID(1, 1, 1), 'name')
check(group == 'cpKeyword', '`fn` highlights as ' .. group)
group = vim.fn.synIDattr(vim.fn.synID(2, 12, 1), 'name')
check(group == 'cpType', '`Int` highlights as ' .. group)

check(vim.wait(10000, function() return #vim.lsp.get_clients({ bufnr = 0, name = 'cpl' }) > 0 end), 'cpl lsp did not attach')
check(vim.wait(10000, function() return #vim.diagnostic.get(0) > 0 end), 'no diagnostic for `let s: Str = n`')
local diagnostic = vim.diagnostic.get(0)[1]
check(diagnostic.lnum == 2, 'the diagnostic is on line ' .. diagnostic.lnum)

local params = { textDocument = { uri = vim.uri_from_fname(file) }, position = { line = 3, character = 10 } }
local replies = vim.lsp.buf_request_sync(0, 'textDocument/hover', params, 5000) or {}
local hover = ''
for _, reply in pairs(replies) do
    if reply.result then hover = reply.result.contents.value end
end
check(hover:find('n: Int', 1, true) ~= nil, 'hover on `n` is ' .. vim.inspect(hover))
local function request(method, line, character)
    local replies = vim.lsp.buf_request_sync(0, method, {
        textDocument = { uri = vim.uri_from_fname(file) },
        position = { line = line, character = character },
    }, 5000) or {}
    for _, reply in pairs(replies) do
        if reply.result then return reply.result end
    end
end
-- Completion at `print` lists the prelude's functions and the locals in scope.
local items = request('textDocument/completion', 3, 6) or {}
local labels = vim.tbl_map(function(item) return item.label end, items.items or items)
check(vim.tbl_contains(labels, 'print') and vim.tbl_contains(labels, 'n'), 'completion is ' .. vim.inspect(labels))
check(vim.bo.omnifunc == 'v:lua.vim.lsp.omnifunc', 'omnifunc is ' .. vim.bo.omnifunc)
-- `gd` on the `n` in `print(n + 1)` jumps to its `let`.
vim.api.nvim_win_set_cursor(0, { 4, 10 })
vim.cmd('normal gd')
check(vim.wait(5000, function() return vim.api.nvim_win_get_cursor(0)[1] == 2 end), 'gd went to ' .. vim.inspect(vim.api.nvim_win_get_cursor(0)))
local lenses = {}
check(vim.wait(10000, function()
    lenses = vim.lsp.codelens.get(0)
    if #lenses == 0 then vim.lsp.codelens.refresh({ bufnr = 0 }) end
    return #lenses > 0
end, 200), 'no code lenses')
local run = lenses[1].command
check(run.title == 'Run on i1' and run.command == 'cpl.exec', 'the first lens is ' .. vim.inspect(run))
check(vim.lsp.commands['cpl.exec'] ~= nil, 'cpl.exec is not a client command')
vim.lsp.commands['cpl.exec'](run, {})
local term = vim.api.nvim_get_current_buf()
check(vim.bo[term].buftype == 'terminal', 'cpl.exec did not open a terminal')
check(vim.wait(10000, function() return vim.v.shell_error ~= nil and vim.b[term].terminal_job_id ~= nil and vim.fn.jobwait({ vim.b[term].terminal_job_id }, 0)[1] ~= -1 end), 'cpl run did not finish')
check(terminal_shows(term, 'Str', run.arguments or {}), 'the run lens showed no `Str` error')

-- `:Cpl check` runs on the file in a terminal split and completes command names.
vim.cmd('wincmd k')
check(vim.api.nvim_buf_get_name(0) == file, 'back in ' .. vim.api.nvim_buf_get_name(0))
local names = vim.fn.getcompletion('Cpl sub', 'cmdline')
check(vim.tbl_contains(names, 'submit'), ':Cpl completes sub to ' .. vim.inspect(names))
vim.cmd('Cpl check')
term = vim.api.nvim_get_current_buf()
check(vim.bo[term].buftype == 'terminal', ':Cpl did not open a terminal')
check(vim.wait(10000, function() return vim.fn.jobwait({ vim.b[term].terminal_job_id }, 0)[1] ~= -1 end), 'cpl check did not finish')
check(terminal_shows(term, 'a.cp:3', { 'check', file }), ':Cpl check showed no `a.cp:3` error')

-- Writing a .cp buffer formats it through the server.
local fmt_file = dir .. '/b.cp'
vim.fn.writefile({ 'let  x=1', 'print( x )' }, fmt_file)
vim.cmd.edit(fmt_file)
check(vim.wait(10000, function() return #vim.lsp.get_clients({ bufnr = 0, name = 'cpl' }) > 0 end), 'cpl lsp did not attach to b.cp')
vim.cmd('write')
local written = vim.fn.readfile(fmt_file)
check(vim.deep_equal(written, { 'let x = 1', 'print(x)' }), 'b.cp was written as ' .. vim.inspect(written))

-- With the parser built, a .cp buffer highlights through tree-sitter and the shared queries.
local parser_dir = dir .. '/ts'
vim.fn.mkdir(parser_dir .. '/parser', 'p')
local grammar = repo .. '/editors/tree-sitter-cp'
local cc = vim.system({ 'cc', '-O2', '-shared', '-fPIC', '-I', grammar .. '/src', grammar .. '/src/parser.c',
    '-o', parser_dir .. '/parser/cp.so' }):wait()
check(cc.code == 0, 'the parser does not build: ' .. (cc.stderr or ''))
vim.opt.rtp:prepend(parser_dir)
local ts_file = dir .. '/b.cp'
vim.fn.writefile({ 'fn walk(n: Int) -> Int:', '    xs.sort()', '    print("{n}", MOD)' }, ts_file)
vim.cmd.edit(ts_file)
check(vim.treesitter.highlighter.active[vim.api.nvim_get_current_buf()] ~= nil, 'tree-sitter is not highlighting')
-- Neovim 0.11 parses off the main loop; parse now so the captures are there to read.
vim.treesitter.get_parser(0):parse()
local function captures(row, col)
    return vim.tbl_map(function(c) return c.capture end, vim.treesitter.get_captures_at_pos(0, row, col))
end
for _, want in ipairs({ { 0, 0, 'keyword' }, { 0, 3, 'function' }, { 0, 11, 'type' }, { 1, 8, 'function.method.call' },
    { 2, 10, 'string' }, { 2, 17, 'constant' } }) do
    local got = captures(want[1], want[2])
    check(vim.tbl_contains(got, want[3]), ('(%d, %d) captures %s, not %s'):format(want[1], want[2], vim.inspect(got), want[3]))
end
io.stdout:write('neovim smoke: ok\n')
os.exit(0)
