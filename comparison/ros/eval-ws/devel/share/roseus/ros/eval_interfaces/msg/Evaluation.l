;; Auto-generated. Do not edit!


(when (boundp 'eval_interfaces::Evaluation)
  (if (not (find-package "EVAL_INTERFACES"))
    (make-package "EVAL_INTERFACES"))
  (shadow 'Evaluation (find-package "EVAL_INTERFACES")))
(unless (find-package "EVAL_INTERFACES::EVALUATION")
  (make-package "EVAL_INTERFACES::EVALUATION"))

(in-package "ROS")
;;//! \htmlinclude Evaluation.msg.html


(defclass eval_interfaces::Evaluation
  :super ros::object
  :slots (_emitter_ts ))

(defmethod eval_interfaces::Evaluation
  (:init
   (&key
    ((:emitter_ts __emitter_ts) 0)
    )
   (send-super :init)
   (setq _emitter_ts (round __emitter_ts))
   self)
  (:emitter_ts
   (&optional __emitter_ts)
   (if __emitter_ts (setq _emitter_ts __emitter_ts)) _emitter_ts)
  (:serialization-length
   ()
   (+
    ;; uint64 _emitter_ts
    8
    ))
  (:serialize
   (&optional strm)
   (let ((s (if strm strm
              (make-string-output-stream (send self :serialization-length)))))
     ;; uint64 _emitter_ts
#+(or :alpha :irix6 :x86_64)
       (progn (sys::poke _emitter_ts (send s :buffer) (send s :count) :long) (incf (stream-count s) 8))
#-(or :alpha :irix6 :x86_64)
       (cond ((and (class _emitter_ts) (= (length (_emitter_ts . bv)) 2)) ;; bignum
              (write-long (ash (elt (_emitter_ts . bv) 0) 0) s)
              (write-long (ash (elt (_emitter_ts . bv) 1) -1) s))
             ((and (class _emitter_ts) (= (length (_emitter_ts . bv)) 1)) ;; big1
              (write-long (elt (_emitter_ts . bv) 0) s)
              (write-long (if (>= _emitter_ts 0) 0 #xffffffff) s))
             (t                                         ;; integer
              (write-long _emitter_ts s)(write-long (if (>= _emitter_ts 0) 0 #xffffffff) s)))
     ;;
     (if (null strm) (get-output-stream-string s))))
  (:deserialize
   (buf &optional (ptr- 0))
   ;; uint64 _emitter_ts
     
#+(or :alpha :irix6 :x86_64)
      (setf _emitter_ts (prog1 (sys::peek buf ptr- :long) (incf ptr- 8)))
#-(or :alpha :irix6 :x86_64)
      (setf _emitter_ts (let ((b0 (prog1 (sys::peek buf ptr- :integer) (incf ptr- 4)))
                  (b1 (prog1 (sys::peek buf ptr- :integer) (incf ptr- 4))))
              (cond ((= b1 -1) b0)
                     ((and (= b1  0)
                           (<= lisp::most-negative-fixnum b0 lisp::most-positive-fixnum))
                      b0)
                    ((= b1  0) (make-instance bignum :size 1 :bv (integer-vector b0)))
                    (t (make-instance bignum :size 2 :bv (integer-vector b0 (ash b1 1)))))))
   ;;
   self)
  )

(setf (get eval_interfaces::Evaluation :md5sum-) "476ebe5b5ffd3d3a307606dab9b18299")
(setf (get eval_interfaces::Evaluation :datatype-) "eval_interfaces/Evaluation")
(setf (get eval_interfaces::Evaluation :definition-)
      "uint64 emitter_ts

")



(provide :eval_interfaces/Evaluation "476ebe5b5ffd3d3a307606dab9b18299")


