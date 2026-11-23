" `cpl lsp` in Vim through the vim-lsp `cpl setup vim` installs: a wrong file
" gets the server's error, and ^X^O completes the file's names.
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
    if &omnifunc !=# 'lsp#complete' || maparg('gd', 'n') !~# 'lsp-definition' || maparg('K', 'n') !~# 'lsp-hover'
        return s:Finish('omnifunc=' . &omnifunc . ' gd=' . maparg('gd', 'n') . ' K=' . maparg('K', 'n'))
    endif
    " ^X^O after `tot` in `print(tot)`, as a contestant types it.
    call cursor(4, 14)
    call feedkeys("i\<C-x>\<C-o>", 'nt')
    let s:tries = 0
    call timer_start(250, function('s:Menu'), {'repeat': -1})
endfunction

function! s:Menu(timer) abort
    let s:tries += 1
    let labels = map(copy(complete_info(['items']).items), {_, item -> item.word})
    if index(labels, 'total') >= 0
        call timer_stop(a:timer)
        return s:Finish('ok')
    endif
    if s:tries > 240
        call timer_stop(a:timer)
        call s:Finish('^X^O offered ' . string(labels) . ' in mode ' . mode())
    endif
endfunction

call timer_start(250, function('s:Check'), {'repeat': -1})
