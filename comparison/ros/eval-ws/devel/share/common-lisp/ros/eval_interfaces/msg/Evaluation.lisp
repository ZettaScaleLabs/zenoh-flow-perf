; Auto-generated. Do not edit!


(cl:in-package eval_interfaces-msg)


;//! \htmlinclude Evaluation.msg.html

(cl:defclass <Evaluation> (roslisp-msg-protocol:ros-message)
  ((emitter_ts
    :reader emitter_ts
    :initarg :emitter_ts
    :type cl:integer
    :initform 0))
)

(cl:defclass Evaluation (<Evaluation>)
  ())

(cl:defmethod cl:initialize-instance :after ((m <Evaluation>) cl:&rest args)
  (cl:declare (cl:ignorable args))
  (cl:unless (cl:typep m 'Evaluation)
    (roslisp-msg-protocol:msg-deprecation-warning "using old message class name eval_interfaces-msg:<Evaluation> is deprecated: use eval_interfaces-msg:Evaluation instead.")))

(cl:ensure-generic-function 'emitter_ts-val :lambda-list '(m))
(cl:defmethod emitter_ts-val ((m <Evaluation>))
  (roslisp-msg-protocol:msg-deprecation-warning "Using old-style slot reader eval_interfaces-msg:emitter_ts-val is deprecated.  Use eval_interfaces-msg:emitter_ts instead.")
  (emitter_ts m))
(cl:defmethod roslisp-msg-protocol:serialize ((msg <Evaluation>) ostream)
  "Serializes a message object of type '<Evaluation>"
  (cl:write-byte (cl:ldb (cl:byte 8 0) (cl:slot-value msg 'emitter_ts)) ostream)
  (cl:write-byte (cl:ldb (cl:byte 8 8) (cl:slot-value msg 'emitter_ts)) ostream)
  (cl:write-byte (cl:ldb (cl:byte 8 16) (cl:slot-value msg 'emitter_ts)) ostream)
  (cl:write-byte (cl:ldb (cl:byte 8 24) (cl:slot-value msg 'emitter_ts)) ostream)
  (cl:write-byte (cl:ldb (cl:byte 8 32) (cl:slot-value msg 'emitter_ts)) ostream)
  (cl:write-byte (cl:ldb (cl:byte 8 40) (cl:slot-value msg 'emitter_ts)) ostream)
  (cl:write-byte (cl:ldb (cl:byte 8 48) (cl:slot-value msg 'emitter_ts)) ostream)
  (cl:write-byte (cl:ldb (cl:byte 8 56) (cl:slot-value msg 'emitter_ts)) ostream)
)
(cl:defmethod roslisp-msg-protocol:deserialize ((msg <Evaluation>) istream)
  "Deserializes a message object of type '<Evaluation>"
    (cl:setf (cl:ldb (cl:byte 8 0) (cl:slot-value msg 'emitter_ts)) (cl:read-byte istream))
    (cl:setf (cl:ldb (cl:byte 8 8) (cl:slot-value msg 'emitter_ts)) (cl:read-byte istream))
    (cl:setf (cl:ldb (cl:byte 8 16) (cl:slot-value msg 'emitter_ts)) (cl:read-byte istream))
    (cl:setf (cl:ldb (cl:byte 8 24) (cl:slot-value msg 'emitter_ts)) (cl:read-byte istream))
    (cl:setf (cl:ldb (cl:byte 8 32) (cl:slot-value msg 'emitter_ts)) (cl:read-byte istream))
    (cl:setf (cl:ldb (cl:byte 8 40) (cl:slot-value msg 'emitter_ts)) (cl:read-byte istream))
    (cl:setf (cl:ldb (cl:byte 8 48) (cl:slot-value msg 'emitter_ts)) (cl:read-byte istream))
    (cl:setf (cl:ldb (cl:byte 8 56) (cl:slot-value msg 'emitter_ts)) (cl:read-byte istream))
  msg
)
(cl:defmethod roslisp-msg-protocol:ros-datatype ((msg (cl:eql '<Evaluation>)))
  "Returns string type for a message object of type '<Evaluation>"
  "eval_interfaces/Evaluation")
(cl:defmethod roslisp-msg-protocol:ros-datatype ((msg (cl:eql 'Evaluation)))
  "Returns string type for a message object of type 'Evaluation"
  "eval_interfaces/Evaluation")
(cl:defmethod roslisp-msg-protocol:md5sum ((type (cl:eql '<Evaluation>)))
  "Returns md5sum for a message object of type '<Evaluation>"
  "476ebe5b5ffd3d3a307606dab9b18299")
(cl:defmethod roslisp-msg-protocol:md5sum ((type (cl:eql 'Evaluation)))
  "Returns md5sum for a message object of type 'Evaluation"
  "476ebe5b5ffd3d3a307606dab9b18299")
(cl:defmethod roslisp-msg-protocol:message-definition ((type (cl:eql '<Evaluation>)))
  "Returns full string definition for message of type '<Evaluation>"
  (cl:format cl:nil "uint64 emitter_ts~%~%~%"))
(cl:defmethod roslisp-msg-protocol:message-definition ((type (cl:eql 'Evaluation)))
  "Returns full string definition for message of type 'Evaluation"
  (cl:format cl:nil "uint64 emitter_ts~%~%~%"))
(cl:defmethod roslisp-msg-protocol:serialization-length ((msg <Evaluation>))
  (cl:+ 0
     8
))
(cl:defmethod roslisp-msg-protocol:ros-message-to-list ((msg <Evaluation>))
  "Converts a ROS message object to a list"
  (cl:list 'Evaluation
    (cl:cons ':emitter_ts (emitter_ts msg))
))
