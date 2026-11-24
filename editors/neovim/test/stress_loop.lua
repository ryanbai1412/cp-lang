-- nvim --headless --clean -l editors/neovim/test/stress_loop.lua CPL REPO
-- The debugger's wrong-answer loop from Neovim: the knapsack demo with its bug, the
-- server's Stress lens run by the server finds and saves a small failing case,
-- `:Cpl test` fails on it, the fix is written, and `:Cpl test` and `:Cpl stress` pass.
local cpl, repo = arg[1], arg[2]
vim.g.cpl_path = cpl

local function check(ok, what)
    if not ok then
        io.stderr:write('neovim stress loop: ' .. what .. '\n')
        os.exit(1)
    end
end

--- The current buffer's code lenses.
local function code_lenses()
    return vim.tbl_map(function(found) return found.lens end, vim.lsp.codelens.get({ bufnr = 0 }))
end

-- What the server shows (`window/showMessage`), and its answer to each run it was asked for.
local messages, ran = {}, {}
vim.notify = function(message) table.insert(messages, message) end
vim.lsp.handlers['workspace/executeCommand'] = function(err, result)
    table.insert(ran, result or { err = err })
end

--- Runs `start`, waits for the server's answer to the run it asks for, and returns
--- that answer and what the server showed meanwhile.
local function server_runs(what, start)
    messages, ran = {}, {}
    start()
    check(vim.wait(60000, function() return #ran > 0 end, 100), what .. ' did not finish')
    return ran[1], table.concat(messages, '\n')
end

vim.opt.runtimepath:prepend(repo .. '/editors/neovim')
vim.cmd('runtime! plugin/**/*.vim plugin/**/*.lua ftdetect/*.vim')
vim.cmd('filetype plugin on')

local dir = vim.fn.tempname()
vim.fn.mkdir(dir, 'p')
local file = dir .. '/k.cp'
local planted = vim.fn.readfile(repo .. '/examples/debugging/knapsack.cp')
vim.fn.writefile(planted, file)
vim.cmd.edit(file)
check(vim.wait(10000, function() return #vim.lsp.get_clients({ bufnr = 0, name = 'cpl' }) > 0 end), 'cpl lsp did not attach')

--- What `start` runs ends with: its exit code and what the server showed and answered.
local function finished(what, start)
    local answer, said = server_runs(what, start)
    return answer.code, said .. '\n' .. vim.inspect(answer.view)
end

local lenses = {}
check(vim.wait(10000, function()
    lenses = code_lenses()
    return #lenses > 0
end, 200), 'no code lenses')
local stress
for _, lens in ipairs(lenses) do
    if lens.command and lens.command.title:find('^Stress') then stress = lens.command end
end
check(stress ~= nil and stress.command == 'cpl.exec', 'no Stress lens in ' .. vim.inspect(vim.tbl_map(function(l) return l.command and l.command.title end, lenses)))
local cpl_client = vim.lsp.get_clients({ bufnr = 0, name = 'cpl' })[1]
local code, shown = finished('the Stress lens', function() cpl_client:exec_cmd(stress, { bufnr = 0 }) end)
check(code == 1, 'the Stress lens exited ' .. code .. ':\n' .. shown)
-- readdir, not glob: Vim's glob misreads the mixed separators of a Windows temp path.
local saved = vim.tbl_filter(function(name) return name:match('^stress%-.*%.in$') ~= nil end,
    vim.fn.readdir(dir .. '/k.tests'))
check(#saved == 1, 'the Stress lens saved ' .. vim.inspect(saved) .. ':\n' .. shown)

code, shown = finished(':Cpl test', function() vim.cmd('Cpl test') end)
check(code ~= 0, ':Cpl test passed the saved case:\n' .. shown)

local fixed = {}
for _, line in ipairs(planted) do
    table.insert(fixed, (line:gsub('for c in 0%.%.=cap:', 'for c in (0..=cap).rev:')))
end
check(table.concat(fixed) ~= table.concat(planted), 'the planted bug moved')
vim.api.nvim_buf_set_lines(0, 0, -1, false, fixed)
vim.cmd('write')

code, shown = finished(':Cpl test after the fix', function() vim.cmd('Cpl test') end)
check(code == 0, ':Cpl test after the fix exited ' .. code .. ':\n' .. shown)
code, shown = finished(':Cpl stress after the fix', function() vim.cmd('Cpl stress') end)
check(code == 0, ':Cpl stress after the fix exited ' .. code .. ':\n' .. shown)
-- `cpl locate` names the first wrong call of a part, and the file shows it on that line.
local part = dir .. '/p.cp'
vim.fn.writefile(vim.fn.readfile(repo .. '/crates/cpl-cli/tests/debugger/part_planted.cp'), part)
vim.fn.writefile({ '1' }, dir .. '/one.in')
vim.cmd.edit(part)
local source = vim.api.nvim_get_current_buf()
code, shown = finished(':Cpl locate', function() vim.cmd('Cpl locate ' .. vim.fn.fnameescape(dir .. '/one.in')) end)
check(code == 1, ':Cpl locate exited ' .. code .. ':\n' .. shown)
local marks = {}
check(vim.wait(5000, function()
    marks = vim.tbl_filter(function(d) return d.message:find('first wrong call', 1, true) ~= nil end,
        vim.diagnostic.get(source))
    return #marks > 0
end), 'no first-wrong-call diagnostic after :Cpl locate:\n' .. shown .. vim.inspect(vim.diagnostic.get(source)))
check(marks[1].lnum == 3 and marks[1].message:find('first wrong call: `tri(k: 1)`', 1, true) ~= nil,
    'the diagnostic is ' .. vim.inspect(marks[1]))
print('neovim stress loop: ok')
vim.cmd('qall!')
