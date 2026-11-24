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

;; The highlighting and syntax come from `cpl setup emacs --grammar', generated from the lexer's tables.
(defvar cp--grammar nil "The grammar cpl printed, once read.")

(defun cp--rules (key)
  "KEY of the grammar cpl generates: `:font-lock', `:syntax' or `:syntactic'."
  (plist-get (or cp--grammar
                 (setq cp--grammar
                       (with-temp-buffer
                         (when (eql 0 (ignore-errors (call-process cp-cpl-program nil '(t nil) nil "setup" "emacs" "--grammar")))
                           (goto-char (point-min))
                           (ignore-errors (read (current-buffer)))))))
             key))

(defun cp-font-lock-keywords ()
  "The font-lock rules cpl generates; with `cpl lsp' running, its semantic tokens add to them."
  (cp--rules :font-lock))

;;;###autoload
(define-derived-mode cp-mode prog-mode "CP"
  "Major mode for CP Lang."
  (dolist (entry (cp--rules :syntax))
    (modify-syntax-entry (car entry) (cdr entry)))
  (setq-local font-lock-defaults '(cp-font-lock-keywords))
  (setq-local syntax-propertize-function (syntax-propertize-via-font-lock (cp--rules :syntactic)))
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
