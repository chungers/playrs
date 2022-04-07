;;; Project-specific template for Emacs.
;;; The variables here are to be filled in by CMake.
;;;
;;;
;;; load this with M-x 'load-file'


;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; Basic colors, etc.
(set-face-attribute 'default nil :height 140)  ; 140 for 14-point font
(line-number-mode 1)
(column-number-mode 1)
(global-hl-line-mode 1) ;; highlights current line
(set-background-color "black")
(set-foreground-color "white")
(set-cursor-color "DarkBlue")
;; Sets the colors for all frames.
(setq default-frame-alist
      (append default-frame-alist
              '((foreground-color . "white")
                (background-color . "black")
                (cursor-color . "DarkBlue")
                )))
;;(setq default-frame-alist '((width . 140) (height . 10)))
(when window-system (set-frame-size (selected-frame) 120 50))

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; Navigating windows:  Use option-<arrow_key>
(windmove-default-keybindings)
(global-set-key [s-left] 'windmove-left)          ; move to left window
(global-set-key [s-right] 'windmove-right)        ; move to right window
(global-set-key [s-up] 'windmove-up)              ; move to upper window
(global-set-key [s-down] 'windmove-down)          ; move to lower window
(global-set-key [(control c) (c)] 'compile)
(global-set-key [(control c) (r)] 'recompile)
(global-set-key [(control c) (k)] 'kill-compilation)

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; Switching to previous buffer
(defun switch-to-previous-buffer ()
  "Switch to previously open buffer.
Repeated invocations toggle between the two most recently open buffers."
  (interactive)
  (switch-to-buffer (other-buffer (current-buffer) 1)))
(global-set-key (kbd "C-c b") 'switch-to-previous-buffer)

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; Useful for refreshing buffers after git pull
(defun revert-all-buffers ()
    "Refreshes all open buffers from their respective files."
    (interactive)
    (dolist (buf (buffer-list))
      (with-current-buffer buf
        (when (and (buffer-file-name) (not (buffer-modified-p)))
          (revert-buffer t t t) )))
    (message "Refreshed open files.") )

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;; General formatting -- 100 character limits
(defun font-lock-width-keyword (width)
  "Return a font-lock style keyword for a string beyond width WIDTH
that uses 'font-lock-warning-face'."
  `((,(format "^%s\\(.+\\)" (make-string width ?.))
     (1 font-lock-warning-face t))))

(font-lock-add-keywords 'c-mode (font-lock-width-keyword 100))
(font-lock-add-keywords 'c++-mode (font-lock-width-keyword 100))
(font-lock-add-keywords 'java-mode (font-lock-width-keyword 100))
(font-lock-add-keywords 'python-mode (font-lock-width-keyword 100))
(font-lock-add-keywords 'js-mode (font-lock-width-keyword 100))
(font-lock-add-keywords 'go-mode (font-lock-width-keyword 100))
(font-lock-add-keywords 'rust-mode (font-lock-width-keyword 100))
(custom-set-faces
   '(my-tab-face            ((((class color)) (:foreground "blue" :underline t))) t)
   '(my-trailing-space-face ((((class color)) (:background "red"))) t)
   '(my-long-line-face ((((class color)) (:background "blue" :underline t))) t))
(add-hook 'font-lock-mode-hook
            (function
             (lambda ()
               (setq font-lock-keywords
                     (append font-lock-keywords
                          '(("\t+" (0 'my-tab-face t))
                            ("^.\\{121,\\}$" (0 'my-long-line-face t))
                           ("[ \t]+$"      (0 'my-trailing-space-face t))))))))
;; Trailing whitespaces:
(add-hook 'write-file-hooks 'maybe-delete-trailing-whitespace)

(defvar skip-whitespace-check nil
  "If non-nil, inhibits behaviour of
  `maybe-delete-trailing-whitespace', which is typically a
  write-file-hook.  This variable may be buffer-local, to permit
  extraneous whitespace on a per-file basis.")
(make-variable-buffer-local 'skip-whitespace-check)

(defun buffer-whitespace-normalized-p ()
  "Returns non-nil if the current buffer contains no tab characters
nor trailing whitespace.  This predicate is useful for determining
whether to enable automatic whitespace normalization.  Simply applying
it blindly to other people's files can cause enormously messy diffs!"
  (save-excursion
    (not  (or (progn (beginning-of-buffer)
                     (search-forward "\t" nil t))
              (progn (beginning-of-buffer)
                     (re-search-forward " +$" nil t))))))

(defun whitespace-check-find-file-hook ()
  (unless (buffer-whitespace-normalized-p)
    (message "Disabling whitespace normalization for this buffer...")
    (setq skip-whitespace-check t)))

;; Install hook so we don't accidentally normalise non-normal files.
(setq find-file-hooks
      (cons #'whitespace-check-find-file-hook find-file-hooks))

(defun toggle-whitespace-removal ()
  "Toggle the value of `skip-whitespace-check' in this buffer."
  (interactive)
  (setq skip-whitespace-check (not skip-whitespace-check))
  (message "Whitespace trimming %s"
           (if skip-whitespace-check "disabled" "enabled")))

(defun maybe-delete-trailing-whitespace ()
  "Calls `delete-trailing-whitespace' iff buffer-local variable
 skip-whitespace-check is nil.  Returns nil."
  (or skip-whitespace-check
      (delete-trailing-whitespace))
  nil)

;;; Use "%" to jump to the matching parenthesis.
(defun goto-match-paren (arg)
  "Go to the matching parenthesis if on parenthesis, otherwise insert
  the character typed."
  (interactive "p")
  (cond ((looking-at "\\s\(") (forward-list 1) (backward-char 1))
    ((looking-at "\\s\)") (forward-char 1) (backward-list 1))
    (t                    (self-insert-command (or arg 1))) ))
(global-set-key "%" `goto-match-paren)



;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;; Top level path
;; Start emacs from the project top level directory:
(setq top-path (getenv "PWD"))

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;; git mode
(load (concat top-path "/hack/emacs/git.el"))
(require 'git)

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;; go mode
(load (concat top-path "/hack/emacs/go-mode.el"))
(load (concat top-path "/hack/emacs/go-mode-autoloads.el"))
;;(load (concat top-path "/hack/emacs/oracle.el"))
(load (concat top-path "/hack/emacs/go-guru.el"))
(require 'go-mode-autoloads)
(require 'go-guru)
(go-guru-hl-identifier-mode)
(add-hook 'go-mode-hook #'go-guru-hl-identifier-mode)
(add-hook 'before-save-hook #'gofmt-before-save)
(add-hook 'go-mode-hook '(lambda ()
  (local-set-key (kbd "C-c C-r") 'go-remove-unused-imports)))
(add-hook 'go-mode-hook '(lambda ()
  (local-set-key (kbd "C-c C-g") 'go-goto-imports)))
(add-hook 'go-mode-hook '(lambda ()
  (local-set-key (kbd "C-c C-k") 'godoc)))
(add-hook 'go-mode-hook '(lambda ()
  (local-set-key (kbd "C-c i") 'go-guru-definition)))
(add-hook 'go-mode-hook '(lambda ()
  (local-set-key (kbd "C-c l") 'go-guru-callers)))
(add-hook 'go-mode-hook '(lambda ()
  (local-set-key (kbd "C-c m") 'go-guru-implements)))

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;; yaml
(load (concat top-path "/hack/emacs/yaml-mode.el"))
(require 'yaml-mode)
    (add-to-list 'auto-mode-alist '("\\.yml$" . yaml-mode))
(add-hook 'yaml-mode-hook
      '(lambda ()
        (define-key yaml-mode-map "\C-m" 'newline-and-indent)))
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;; rust mode
(add-to-list 'load-path (concat top-path "/hack/lib/github.com/rust-lang/rust-mode/"))
(autoload 'rust-mode "rust-mode" nil t)
(require 'rust-mode)
    (add-to-list 'auto-mode-alist '("\\.rs" . rust-mode))

(font-lock-add-keywords 'rust-mode (font-lock-width-keyword 80))
(add-hook 'rust-mode-hook
          (lambda () (setq indent-tabs-mode nil)))
(add-hook 'rust-mode-hook
          (lambda () (prettify-symbols-mode)))
(define-key rust-mode-map (kbd "C-c C-c") 'rust-run)
(setq rust-format-on-save t)



;; Compilation mode hook
(defun my-compile-hook ()
  ;; compile - go to error
  (local-set-key "\C-cg" 'compile-goto-error)
  (local-set-key "\C-cn" 'compilation-next-error)
  (local-set-key "\C-cp" 'compilation-previous-error)
)
(add-hook 'compilation-mode-hook 'my-compile-hook)


(defun my-compilation-hook ()
  (when (not (get-buffer-window "*compilation*"))
    (save-selected-window
      (save-excursion
        (let* ((w (split-window-vertically))
               (h (window-height w)))
          (select-window w)
          (switch-to-buffer "*compilation*")
          (shrink-window (- h 20)))))))
;;(add-hook 'compilation-mode-hook 'my-compilation-hook)

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;; javascript
(defun my-javascript-mode-hook ()
  (setq indent-tabs-mode nil tab-width 4 js-indent-level 4)
  )
(add-hook 'javascript-mode-hook 'my-javascript-mode-hook)
