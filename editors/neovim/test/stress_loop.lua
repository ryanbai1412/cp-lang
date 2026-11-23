-- nvim --headless --clean -l editors/neovim/test/stress_loop.lua CPL REPO
-- The debugger's wrong-answer loop from Neovim: the knapsack demo with its bug, the
-- server's Stress lens run through `cpl.exec` finds and saves a small failing case,
-- `:Cpl test` fails on it, the fix is written, and `:Cpl test` and `:Cpl stress` pass.
local cpl, repo = arg[1], arg[2]
vim.g.cpl_path = cpl

local function check(ok, what)
    if not ok then
        io.stderr:write('neovim stress loop: ' .. what .. '\n')
        os.exit(1)
    end
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

-- A finished terminal's job may already be gone for `jobwait`; TermClose keeps its status.
local status = {}
vim.api.nvim_create_autocmd('TermClose', {
    callback = function(ev) status[ev.buf] = vim.v.event.status end,
})

-- Waits for the terminal a command opened, returns its exit code and text, and goes back.
local function finished(what)
    local term = vim.api.nvim_get_current_buf()
    check(vim.bo[term].buftype == 'terminal', what .. ' did not open a terminal')
    check(vim.wait(60000, function() return status[term] ~= nil end), what .. ' did not finish')
    local code = status[term]
    -- The last of the output can reach the buffer after TermClose (Windows' ConPTY).
    local shown = ''
    vim.wait(5000, function()
        shown = table.concat(vim.api.nvim_buf_get_lines(term, 0, -1, false), '\n')
        return shown:find('%S') ~= nil
    end)
    vim.cmd('wincmd p')
    return code, shown
end

local lenses = {}
check(vim.wait(10000, function()
    lenses = vim.lsp.codelens.get(0)
    if #lenses == 0 then vim.lsp.codelens.refresh({ bufnr = 0 }) end
    return #lenses > 0
end, 200), 'no code lenses')
local stress
for _, lens in ipairs(lenses) do
    if lens.command and lens.command.title:find('^Stress') then stress = lens.command end
end
check(stress ~= nil and stress.command == 'cpl.exec', 'no Stress lens in ' .. vim.inspect(vim.tbl_map(function(l) return l.command and l.command.title end, lenses)))
vim.lsp.commands['cpl.exec'](stress)
local code, shown = finished('the Stress lens')
check(code == 1, 'the Stress lens exited ' .. code .. ':\n' .. shown)
local saved = vim.fn.glob(dir .. '/k.tests/stress-*.in', false, true)
check(#saved == 1, 'the Stress lens saved ' .. vim.inspect(saved) .. ':\n' .. shown)

vim.cmd('Cpl test')
code, shown = finished(':Cpl test')
check(code ~= 0, ':Cpl test passed the saved case:\n' .. shown)

local fixed = {}
for _, line in ipairs(planted) do
    table.insert(fixed, (line:gsub('for c in 0%.%.=cap:', 'for c in (0..=cap).rev:')))
end
check(table.concat(fixed) ~= table.concat(planted), 'the planted bug moved')
vim.api.nvim_buf_set_lines(0, 0, -1, false, fixed)
vim.cmd('write')

vim.cmd('Cpl test')
code, shown = finished(':Cpl test after the fix')
check(code == 0, ':Cpl test after the fix exited ' .. code .. ':\n' .. shown)
vim.cmd('Cpl stress')
code, shown = finished(':Cpl stress after the fix')
check(code == 0, ':Cpl stress after the fix exited ' .. code .. ':\n' .. shown)
print('neovim stress loop: ok')
vim.cmd('qall!')
