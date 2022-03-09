;; Auto-generated. Do not edit!


(when (boundp 'eval_interfaces::Thr)
  (if (not (find-package "EVAL_INTERFACES"))
    (make-package "EVAL_INTERFACES"))
  (shadow 'Thr (find-package "EVAL_INTERFACES")))
(unless (find-package "EVAL_INTERFACES::THR")
  (make-package "EVAL_INTERFACES::THR"))

(in-package "ROS")
;;//! \htmlinclude Thr.msg.html


(defclass eval_interfaces::Thr
  :super ros::object
  :slots (_payload ))

(defmethod eval_interfaces::Thr
  (:init
   (&key
    ((:payload __payload) (make-array 0 :initial-element 0 :element-type :integer))
    )
   (send-super :init)
   (setq _payload __payload)
   self)
  (:payload
   (&optional __payload)
   (if __payload (setq _payload __payload)) _payload)
  (:serialization-length
   ()
   (+
    ;; byte[] _payload
    (* 1    (length _payload)) 4
    ))
  (:serialize
   (&optional strm)
   (let ((s (if strm strm
              (make-string-output-stream (send self :serialization-length)))))
     ;; byte[] _payload
     (write-long (length _payload) s)
     (dotimes (i (length _payload))
       (write-byte (elt _payload i) s)
       )
     ;;
     (if (null strm) (get-output-stream-string s))))
  (:deserialize
   (buf &optional (ptr- 0))
   ;; byte[] _payload
   (let (n)
     (setq n (sys::peek buf ptr- :integer)) (incf ptr- 4)
     (setq _payload (instantiate integer-vector n))
     (dotimes (i n)
     (setf (elt _payload i) (sys::peek buf ptr- :char)) (incf ptr- 1)
     (if (> (elt _payload i) 127) (setf (elt _payload i) (- (elt _payload i) 256)))
     ))
   ;;
   self)
  )

(setf (get eval_interfaces::Thr :md5sum-) "879bb387136bcc2d41e0b6899f433588")
(setf (get eval_interfaces::Thr :datatype-) "eval_interfaces/Thr")
(setf (get eval_interfaces::Thr :definition-)
      "byte[] payload

")



(provide :eval_interfaces/Thr "879bb387136bcc2d41e0b6899f433588")


