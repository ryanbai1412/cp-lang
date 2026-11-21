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

;; The reserved words of `TokenKind::keyword' in crates/cpl-syntax/src/token.rs.
(defconst cp-keywords
  '("let" "var" "fn" "struct" "enum" "domain" "type" "const" "trait" "impl" "use" "pub"
    "if" "elif" "else" "for" "while" "in" "match" "return" "break" "continue"
    "and" "or" "not" "assert" "mut" "self" "lends" "yield" "unchecked"))

(defconst cp-constants '("true" "false" "none"))

(defconst cp-font-lock-keywords
  `((,(regexp-opt cp-keywords 'symbols) . font-lock-keyword-face)
    (,(regexp-opt cp-constants 'symbols) . font-lock-constant-face)
    ("@[[:alpha:]_][[:alnum:]_]*" . font-lock-preprocessor-face)
    ("\\_<fn[ \t]+\\([[:alpha:]_][[:alnum:]_]*\\)" 1 font-lock-function-name-face)
    ("\\_<[[:upper:]][[:alnum:]_]*\\_>" . font-lock-type-face)
    ("'[[:alpha:]_][[:alnum:]_]*\\_>\\(?:[^']\\|$\\)" . font-lock-constant-face)
    ("\\_<\\(?:0[xX][[:xdigit:]_]+\\|0[bB][01_]+\\|[0-9][0-9_]*\\(?:\\.[0-9][0-9_]*\\)?\\(?:[eE][+-]?[0-9][0-9_]*\\)?\\)\\_>"
     . font-lock-constant-face)))

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
