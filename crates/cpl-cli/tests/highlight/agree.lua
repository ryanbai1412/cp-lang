-- The Vim syntax group at each token start the test asks about
-- (crates/cpl-cli/tests/highlight_agrees.rs), with `cpl setup vim --grammar`'s file.
--   nvim --headless --clean -l agree.lua QUERIES.json OUT.json RUNTIME_DIR
local queries_path, out_path, runtime = arg[1], arg[2], arg[3]
vim.opt.runtimepath:prepend(runtime)
vim.cmd('syntax enable')
local out = {}
for _, q in ipairs(vim.json.decode(table.concat(vim.fn.readfile(queries_path), '\n'))) do
    vim.cmd('silent edit! ++ff=unix ++enc=utf-8 ' .. vim.fn.fnameescape(q.path))
    vim.cmd('setlocal syntax=cp')
    vim.cmd('syntax sync fromstart')
    local names = {}
    for _, at in ipairs(q.at) do
        names[#names + 1] = vim.fn.synIDattr(vim.fn.synID(at[1] + 1, at[2] + 1, 1), 'name')
    end
    out[#out + 1] = names
    vim.cmd('bwipeout!')
end
vim.fn.writefile({ vim.json.encode(out) }, out_path)
