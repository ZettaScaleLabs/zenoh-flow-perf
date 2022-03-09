; Auto-generated. Do not edit!


(cl:in-package eval_interfaces-msg)


;//! \htmlinclude Thr.msg.html

(cl:defclass <Thr> (roslisp-msg-protocol:ros-message)
  ((payload
    :reader payload
    :initarg :payload
    :type (cl:vector cl:integer)
   :initform (cl:make-array 0 :element-type 'cl:integer :initial-element 0)))
)

(cl:defclass Thr (<Thr>)
  ())

(cl:defmethod cl:initialize-instance :after ((m <Thr>) cl:&rest args)
  (cl:declare (cl:ignorable args))
  (cl:unless (cl:typep m 'Thr)
    (roslisp-msg-protocol:msg-deprecation-warning "using old message class name eval_interfaces-msg:<Thr> is deprecated: use eval_interfaces-msg:Thr instead.")))

(cl:ensure-generic-function 'payload-val :lambda-list '(m))
(cl:defmethod payload-val ((m <Thr>))
  (roslisp-msg-protocol:msg-deprecation-warning "Using old-style slot reader eval_interfaces-msg:payload-val is deprecated.  Use eval_interfaces-msg:payload instead.")
  (payload m))
(cl:defmethod roslisp-msg-protocol:serialize ((msg <Thr>) ostream)
  "Serializes a message object of type '<Thr>"
  (cl:let ((__ros_arr_len (cl:length (cl:slot-value msg 'payload))))
    (cl:write-byte (cl:ldb (cl:byte 8 0) __ros_arr_len) ostream)
    (cl:write-byte (cl:ldb (cl:byte 8 8) __ros_arr_len) ostream)
    (cl:write-byte (cl:ldb (cl:byte 8 16) __ros_arr_len) ostream)
    (cl:write-byte (cl:ldb (cl:byte 8 24) __ros_arr_len) ostream))
  (cl:map cl:nil #'(cl:lambda (ele) (cl:write-byte (cl:ldb (cl:byte 8 0) ele) ostream))
   (cl:slot-value msg 'payload))
)
(cl:defmethod roslisp-msg-protocol:deserialize ((msg <Thr>) istream)
  "Deserializes a message object of type '<Thr>"
  (cl:let ((__ros_arr_len 0))
    (cl:setf (cl:ldb (cl:byte 8 0) __ros_arr_len) (cl:read-byte istream))
    (cl:setf (cl:ldb (cl:byte 8 8) __ros_arr_len) (cl:read-byte istream))
    (cl:setf (cl:ldb (cl:byte 8 16) __ros_arr_len) (cl:read-byte istream))
    (cl:setf (cl:ldb (cl:byte 8 24) __ros_arr_len) (cl:read-byte istream))
  (cl:setf (cl:slot-value msg 'payload) (cl:make-array __ros_arr_len))
  (cl:let ((vals (cl:slot-value msg 'payload)))
    (cl:dotimes (i __ros_arr_len)
    (cl:setf (cl:ldb (cl:byte 8 0) (cl:aref vals i)) (cl:read-byte istream)))))
  msg
)
(cl:defmethod roslisp-msg-protocol:ros-datatype ((msg (cl:eql '<Thr>)))
  "Returns string type for a message object of type '<Thr>"
  "eval_interfaces/Thr")
(cl:defmethod roslisp-msg-protocol:ros-datatype ((msg (cl:eql 'Thr)))
  "Returns string type for a message object of type 'Thr"
  "eval_interfaces/Thr")
(cl:defmethod roslisp-msg-protocol:md5sum ((type (cl:eql '<Thr>)))
  "Returns md5sum for a message object of type '<Thr>"
  "879bb387136bcc2d41e0b6899f433588")
(cl:defmethod roslisp-msg-protocol:md5sum ((type (cl:eql 'Thr)))
  "Returns md5sum for a message object of type 'Thr"
  "879bb387136bcc2d41e0b6899f433588")
(cl:defmethod roslisp-msg-protocol:message-definition ((type (cl:eql '<Thr>)))
  "Returns full string definition for message of type '<Thr>"
  (cl:format cl:nil "byte[] payload~%~%~%"))
(cl:defmethod roslisp-msg-protocol:message-definition ((type (cl:eql 'Thr)))
  "Returns full string definition for message of type 'Thr"
  (cl:format cl:nil "byte[] payload~%~%~%"))
(cl:defmethod roslisp-msg-protocol:serialization-length ((msg <Thr>))
  (cl:+ 0
     4 (cl:reduce #'cl:+ (cl:slot-value msg 'payload) :key #'(cl:lambda (ele) (cl:declare (cl:ignorable ele)) (cl:+ 1)))
))
(cl:defmethod roslisp-msg-protocol:ros-message-to-list ((msg <Thr>))
  "Converts a ROS message object to a list"
  (cl:list 'Thr
    (cl:cons ':payload (payload msg))
))
