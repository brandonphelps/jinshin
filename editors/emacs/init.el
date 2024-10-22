

(defun insert-hash(arg)
  (interactive "sobj: ")
  (save-excursion (insert arg))
  )


(if 'json-available-p
    (message "json is available")
  (message "Json is not avialable"))

(message "%s" (json-parse-string "\"{'a':1}\""))

(message "%s" (aref (json-parse-string "[1,2,3,4]") 3))

(if (proper-list-p )
    (message "is a list")
  (message "is not a proper list"))
