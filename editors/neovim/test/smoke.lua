-- nvim --headless --clean -l editors/neovim/test/smoke.lua CPL REPO
-- Loads editors/neovim as a plugin, opens a .cp file with a type error and checks the
-- filetype, the highlighting, the server's diagnostic, its hover and the run lens beside
-- the sample `i1`, run through `cpl.exec`.
local cpl, repo = arg[1], arg[2]
vim.opt.rtp:prepend(repo .. '/editors/neovim')
vim.g.cpl_path = cpl
vim.cmd('filetype plugin on')
vim.cmd('syntax on')
vim.cmd('runtime! plugin/cplang.lua')

local function check(ok, what)
    if not ok then
        io.stderr:write('neovim smoke: ' .. what .. '\n')
        os.exit(1)
    end
end

local dir = vim.fn.tempname()
vim.fn.mkdir(dir, 'p')
local file = dir .. '/a.cp'
vim.fn.writefile({ '5' }, dir .. '/i1')
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
local shown = table.concat(vim.api.nvim_buf_get_lines(term, 0, -1, false), '\n')
check(shown:find('Str', 1, true) ~= nil, 'the run lens showed ' .. vim.inspect(shown))
io.stdout:write('neovim smoke: ok\n')
os.exit(0)
