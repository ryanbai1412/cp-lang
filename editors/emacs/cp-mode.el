;;; cp-mode.el --- Major mode for CP Lang -*- lexical-binding: t -*-

;;; Commentary:
;; Highlighting and comments for .cp files, and `cpl lsp' for eglot and lsp-mode.
;; `cpl setup emacs' installs it as a package that runs the cpl setup ran as; then
;;   (add-hook 'cp-mode-hook #'eglot-ensure)   ; or #'lsp
;; Set `cp-cpl-program' to run another cpl.

;;; Code:

(defgroup cp nil "CP Lang." :group 'languages)

(defcustom cp-cpl-program "cpl"
  "The cpl executable; its `lsp' subcommand is the language server."
  :type 'string)

;; The highlighting and syntax: cp-grammar.el, which `cpl setup emacs' generates from the
;; lexer's tables beside this file.
(require 'cp-grammar)

(defun cp-font-lock-keywords ()
  "The font-lock rules cpl generates; with `cpl lsp' running, its semantic tokens add to them."
  (plist-get cp--grammar :font-lock))

;;;###autoload
(define-derived-mode cp-mode prog-mode "CP"
  "Major mode for CP Lang."
  (dolist (entry (plist-get cp--grammar :syntax))
    (modify-syntax-entry (car entry) (cdr entry)))
  (setq-local font-lock-defaults '(cp-font-lock-keywords))
  (setq-local syntax-propertize-function (syntax-propertize-via-font-lock (plist-get cp--grammar :syntactic)))
  (setq-local comment-start "// ")
  (setq-local comment-start-skip "//+[ \t]*")
  (setq-local indent-tabs-mode nil)
  (setq-local tab-width 4))

;; Before Emacs's own `\\.[ch]p\\'' entry, which calls .cp C++.
;;;###autoload
(add-to-list 'auto-mode-alist '("\\.cp\\'" . cp-mode))

(defun cp--server-command (&rest _)
  "The language server command line."
  (list cp-cpl-program "lsp"))

(with-eval-after-load 'eglot
  (add-to-list 'eglot-server-programs '(cp-mode . cp--server-command)))

(with-eval-after-load 'lsp-mode
  (add-to-list 'lsp-language-id-configuration '(cp-mode . "cp"))
  (lsp-register-client
   (make-lsp-client :new-connection (lsp-stdio-connection #'cp--server-command)
                    :activation-fn (lsp-activate-on "cp")
                    :server-id 'cpl)))

(provide 'cp-mode)
;;; cp-mode.el ends here
