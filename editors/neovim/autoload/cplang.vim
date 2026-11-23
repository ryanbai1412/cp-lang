" The `cpl` to run: g:cpl_path, then the PATH, then where the installer
" ($CPL_HOME/bin, by default ~/.cpl/bin) and cargo put it, so an editor
" started from a desktop launcher, without the shell's PATH, finds it too.
function! cplang#path() abort
    if !empty(get(g:, 'cpl_path', ''))
        return g:cpl_path
    endif
    if executable('cpl')
        return 'cpl'
    endif
    let l:exe = has('win32') ? 'cpl.exe' : 'cpl'
    let l:dirs = empty($CPL_HOME) ? [] : [$CPL_HOME . '/bin']
    for l:dir in l:dirs + [expand('~/.cpl/bin'), expand('~/.cargo/bin')]
        if executable(l:dir . '/' . l:exe)
            return l:dir . '/' . l:exe
        endif
    endfor
    return 'cpl'
endfunction
