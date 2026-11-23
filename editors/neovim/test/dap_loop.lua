-- nvim --headless --clean -l editors/neovim/test/dap_loop.lua CPL REPO NVIM_DAP
-- Time travel from Neovim: with nvim-dap installed, the plugin's `cp` adapter launches
-- `cpl dap` on the file, it stops on entry, two steps forward and one back land on the
-- line the first step reached, and the frame shows the loop's `i`.
local cpl, repo, nvim_dap = arg[1], arg[2], arg[3]
vim.g.cpl_path = cpl

local function check(ok, what)
    if not ok then
        io.stderr:write('neovim dap loop: ' .. what .. '\n')
        os.exit(1)
    end
end

vim.opt.runtimepath:prepend(nvim_dap)
vim.opt.runtimepath:prepend(repo .. '/editors/neovim')
vim.cmd('runtime! plugin/**/*.vim plugin/**/*.lua ftdetect/*.vim')
vim.cmd('filetype plugin on')

local dir = vim.fn.tempname()
vim.fn.mkdir(dir, 'p')
local file = dir .. '/d.cp'
vim.fn.writefile({ 'var s = 0', 'for i in 0..3:', '    s += i', 'print(s)' }, file)
vim.cmd.edit(file)
local dap = require('dap')
check(dap.adapters.cp ~= nil, 'the plugin registered no `cp` adapter')

local stops = 0
dap.listeners.after.event_stopped['cplang_test'] = function() stops = stops + 1 end
local function line_after(action, what)
    local before, frame = stops, dap.session() and dap.session().current_frame
    action()
    -- nvim-dap asks for the stack after the stop event: wait for the new frame.
    check(vim.wait(20000, function()
        local s = dap.session()
        return stops > before and s ~= nil and s.current_frame ~= nil and s.current_frame ~= frame
    end, 50), what .. ' did not stop')
    return dap.session().current_frame.line
end

local entry = line_after(function() dap.run(dap.configurations.cp[1]) end, 'launch')
local first = line_after(dap.step_over, 'the first step')
local second = line_after(dap.step_over, 'the second step')
check(first ~= entry or second ~= first, 'stepping did not move: ' .. entry .. ', ' .. first .. ', ' .. second)
local back = line_after(dap.step_back, 'step back')
check(back == first, 'step back went to line ' .. back .. ', not ' .. first)

local session = dap.session()
local names
session:request('scopes', { frameId = session.current_frame.id }, function(_, scopes)
    session:request('variables', { variablesReference = scopes.scopes[1].variablesReference }, function(_, vars)
        names = vim.tbl_map(function(v) return v.name end, vars.variables)
    end)
end)
check(vim.wait(10000, function() return names ~= nil end, 50), 'no variables')
check(vim.tbl_contains(names, 'i'), 'the frame shows ' .. vim.inspect(names))
dap.terminate()
print('neovim dap loop: ok')
vim.cmd('qall!')
