;;; agree.el --- the face cp-mode gives each token start the test asks about  -*- lexical-binding: t -*-
;; crates/cpl-cli/tests/highlight_agrees.rs runs:
;;   emacs --batch -Q -l agree.el QUERIES.json OUT.json CPL
(require 'json)
(let* ((args command-line-args-left)
       (queries (with-temp-buffer
                  (insert-file-contents (nth 0 args))
                  (json-parse-buffer :array-type 'list)))
       (out nil))
  (setq command-line-args-left nil)
  (defvar cp-cpl-program)
  (setq cp-cpl-program (nth 2 args))
  ;; cp-mode as `cpl setup emacs' installs it, with the cp-grammar.el it generates.
  (let ((elpa (make-temp-file "cp-elpa" t)))
    (call-process cp-cpl-program nil nil nil "setup" "emacs" "--dir" elpa)
    (add-to-list 'load-path (car (file-expand-wildcards (expand-file-name "cp-mode-*" elpa)))))
  (require 'cp-mode)
  (dolist (q queries)
    (with-temp-buffer
      (let ((coding-system-for-read 'utf-8-unix))
        (insert-file-contents (gethash "path" q)))
      (cp-mode)
      (font-lock-ensure)
      (push (vconcat
             (mapcar (lambda (at)
                       (let ((face (get-text-property (1+ (nth 3 at)) 'face)))
                         (format "%s" (or (if (consp face) (car face) face) ""))))
                     (gethash "at" q)))
            out)))
  (with-temp-file (nth 1 args)
    (insert (json-encode (vconcat (nreverse out))))))
