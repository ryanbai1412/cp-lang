;;; cp-mode.el --- Major mode for CP Lang -*- lexical-binding: t -*-

;;; Commentary:
;; Highlighting and comments for .cp files, and `cpl lsp' for eglot and lsp-mode:
;;   (add-to-list 'load-path "/path/to/cp-lang/editors/emacs")
;;   (require 'cp-mode)
;;   (add-hook 'cp-mode-hook #'eglot-ensure)   ; or #'lsp
;; Set `cp-cpl-program' when cpl is not on the PATH.

;;; Code:

(defgroup cp nil "CP Lang." :group 'languages)

(defcustom cp-cpl-program "cpl"
  "The cpl executable; its `lsp' subcommand is the language server."
  :type 'string)

;; The rules come from `cpl setup emacs --grammar', generated from the lexer's own tables.
(defvar cp--grammar nil "The font-lock rules cpl printed, once read.")

(defun cp-font-lock-keywords ()
  "The font-lock rules cpl generates; with `cpl lsp' running, its semantic tokens add to them."
  (or cp--grammar
      (setq cp--grammar
            (with-temp-buffer
              (when (eql 0 (ignore-errors (call-process cp-cpl-program nil '(t nil) nil "setup" "emacs" "--grammar")))
                (goto-char (point-min))
                (ignore-errors (read (current-buffer))))))))

(defvar cp-mode-syntax-table
  (let ((table (make-syntax-table)))
    (modify-syntax-entry ?/ ". 12" table)
    (modify-syntax-entry ?\n ">" table)
    (modify-syntax-entry ?_ "_" table)
    (modify-syntax-entry ?' "\"" table)
    (modify-syntax-entry ?\" "\"" table)
    table))

(defun cp--syntax-propertize (start end)
  "Make a quote that starts a loop label, as in 'outer, punctuation between START and END."
  (goto-char start)
  (while (re-search-forward "'[[:alpha:]_][[:alnum:]_]+" end t)
    (unless (eq (char-after) ?')
      (put-text-property (match-beginning 0) (1+ (match-beginning 0)) 'syntax-table (string-to-syntax ".")))))

;;;###autoload
(define-derived-mode cp-mode prog-mode "CP"
  "Major mode for CP Lang."
  (setq-local font-lock-defaults '(cp-font-lock-keywords))
  (setq-local syntax-propertize-function #'cp--syntax-propertize)
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
