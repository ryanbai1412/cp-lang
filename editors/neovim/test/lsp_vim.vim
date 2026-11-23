" `cpl lsp` in Vim through the vim-lsp `cpl setup vim` installs: a wrong file
" gets the server's error and completion offers the file's names.
" smoke_vim.sh runs it with the setup's package on 'packpath'; it writes "ok"
" or what went wrong to g:cpl_result, then quits.
let s:tries = 0

function! s:Finish(message) abort
    call writefile([a:message], g:cpl_result)
    " A `qall!` inside vim-lsp's callback waits for input; a timer's doesn't.
    call timer_start(0, {-> execute('qall!')})
endfunction

function! s:Check(timer) abort
    let s:tries += 1
    if !exists('*lsp#get_buffer_diagnostics_counts')
        return s:tries > 240 ? s:Finish('vim-lsp is not loaded') : 0
    endif
    if lsp#get_buffer_diagnostics_counts().error == 0
        return s:tries > 240 ? s:Finish('no diagnostic from cpl lsp in 60 s') : 0
    endif
    call timer_stop(a:timer)
    call cursor(4, 14)
    call lsp#send_request('cpl', {
        \ 'method': 'textDocument/completion',
        \ 'params': {'textDocument': lsp#get_text_document_identifier(), 'position': lsp#get_position()},
        \ 'on_notification': function('s:Completed')})
    call timer_start(60000, {-> s:Finish('no completion from cpl lsp in 60 s')})
endfunction

function! s:Completed(data) abort
    let result = get(a:data.response, 'result', [])
    let items = type(result) == v:t_dict ? get(result, 'items', []) : result
    let labels = map(copy(items), {_, item -> item.label})
    call s:Finish(index(labels, 'total') >= 0 ? 'ok' : 'completion offered ' . string(labels))
endfunction

call timer_start(250, function('s:Check'), {'repeat': -1})
