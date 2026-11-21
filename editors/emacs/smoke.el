;;; smoke.el --- emacs --batch -l editors/emacs/smoke.el -*- lexical-binding: t -*-
;; Loads cp-mode, opens a .cp file and checks the mode, the highlighting, comments and
;; the eglot registration, then talks to the server that registration starts ($CPL, or
;; cpl on the PATH).
(require 'cl-lib)
(add-to-list 'load-path (file-name-directory load-file-name))
(require 'cp-mode)

(defun cp-smoke-check (ok what)
  (unless ok (message "emacs smoke: %s" what) (kill-emacs 1)))

(let ((file (make-temp-file "cp" nil ".cp" "@memo\nfn main():\n    let n: Int = read()  // c\n    for 'outer i in 0..n:\n        print('x')\n")))
  (find-file file)
  (cp-smoke-check (eq major-mode 'cp-mode) (format "major mode is %s" major-mode))
  (font-lock-ensure)
  (cl-flet ((face-at (needle) (goto-char (point-min)) (search-forward needle) (get-text-property (match-beginning 0) 'face)))
    (cp-smoke-check (eq (face-at "fn") 'font-lock-keyword-face) "`fn` is not a keyword")
    (cp-smoke-check (eq (face-at "main") 'font-lock-function-name-face) "`main` is not a function name")
    (cp-smoke-check (eq (face-at "Int") 'font-lock-type-face) "`Int` is not a type")
    (cp-smoke-check (eq (face-at "@memo") 'font-lock-preprocessor-face) "`@memo` is not a decorator")
    (cp-smoke-check (eq (face-at "// c") 'font-lock-comment-delimiter-face) "`//` does not start a comment")
    (cp-smoke-check (eq (face-at "'outer") 'font-lock-constant-face) "`'outer` is not a label")
    (cp-smoke-check (eq (face-at "'x'") 'font-lock-string-face) "`'x'` is not a char"))
  (goto-char (point-min))
  (comment-line 1)
  (cp-smoke-check (string-prefix-p "// @memo" (buffer-string)) "comment-line does not write //"))

(let ((stub (make-temp-file "eglot" t)))
  (with-temp-file (expand-file-name "eglot.el" stub)
    (insert "(defvar eglot-server-programs nil)\n(provide 'eglot)\n"))
  (add-to-list 'load-path stub)
  (require 'eglot))
(let ((entry (assq 'cp-mode eglot-server-programs)))
  (cp-smoke-check entry "eglot does not know cp-mode")
  (cp-smoke-check (equal (funcall (cdr entry)) '("cpl" "lsp")) "the eglot command is not `cpl lsp`"))
;; The command cp-mode gives eglot and lsp-mode starts a server that answers `initialize'
;; and publishes the error in a file, over jsonrpc.el (what eglot is built on).
(require 'jsonrpc)
(setq cp-cpl-program (or (getenv "CPL") "cpl"))
(let* ((diagnostics nil)
       (process (make-process :name "cpl" :command (cp--server-command) :connection-type 'pipe
                              :coding 'utf-8-emacs-unix :noquery t
                              :stderr (get-buffer-create " *cpl stderr*")))
       (server (jsonrpc-process-connection
                :name "cpl" :process process
                :notification-dispatcher
                (lambda (_server method params)
                  (when (eq method 'textDocument/publishDiagnostics) (setq diagnostics params)))))
       (text "fn main():\n    let n: Int = read()\n    let s: Str = n\n    print(n + 1)\n")
       (uri (concat "file://" (make-temp-file "cp" nil ".cp" text)))
       (capabilities (plist-get (jsonrpc-request server :initialize
                                                 (list :processId (emacs-pid) :rootUri nil
                                                       :capabilities '(:textDocument (:hover (:contentFormat ["markdown"])))))
                                :capabilities)))
  (dolist (want '(:hoverProvider :completionProvider :definitionProvider :renameProvider :codeActionProvider))
    (cp-smoke-check (plist-get capabilities want) (format "initialize lacks %s" want)))
  (jsonrpc-notify server :initialized '(:empty :json-false))
  (jsonrpc-notify server :textDocument/didOpen
                  (list :textDocument (list :uri uri :languageId "cp" :version 1 :text text)))
  (with-timeout (20 (cp-smoke-check nil "no diagnostics published"))
    (while (not diagnostics) (accept-process-output nil 0.05)))
  (let ((lines (mapcar (lambda (d) (plist-get (plist-get (plist-get d :range) :start) :line))
                       (seq-filter (lambda (d) (eql (plist-get d :severity) 1))
                                   (plist-get diagnostics :diagnostics)))))
    (cp-smoke-check (equal lines '(2)) (format "errors on lines %s, not on `let s: Str = n'" lines)))
  (jsonrpc-request server :shutdown nil)
  (jsonrpc-notify server :exit nil)
  (with-timeout (5 (cp-smoke-check nil "the server did not exit"))
    (while (process-live-p process) (accept-process-output nil 0.05)))
  (cp-smoke-check (eql (process-exit-status process) 0)
                  (format "the server exited with %s" (process-exit-status process))))
(message "emacs smoke: ok")
