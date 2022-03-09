// Auto-generated. Do not edit!

// (in-package eval_interfaces.msg)


"use strict";

const _serializer = _ros_msg_utils.Serialize;
const _arraySerializer = _serializer.Array;
const _deserializer = _ros_msg_utils.Deserialize;
const _arrayDeserializer = _deserializer.Array;
const _finder = _ros_msg_utils.Find;
const _getByteLength = _ros_msg_utils.getByteLength;

//-----------------------------------------------------------

class Evaluation {
  constructor(initObj={}) {
    if (initObj === null) {
      // initObj === null is a special case for deserialization where we don't initialize fields
      this.emitter_ts = null;
    }
    else {
      if (initObj.hasOwnProperty('emitter_ts')) {
        this.emitter_ts = initObj.emitter_ts
      }
      else {
        this.emitter_ts = 0;
      }
    }
  }

  static serialize(obj, buffer, bufferOffset) {
    // Serializes a message object of type Evaluation
    // Serialize message field [emitter_ts]
    bufferOffset = _serializer.uint64(obj.emitter_ts, buffer, bufferOffset);
    return bufferOffset;
  }

  static deserialize(buffer, bufferOffset=[0]) {
    //deserializes a message object of type Evaluation
    let len;
    let data = new Evaluation(null);
    // Deserialize message field [emitter_ts]
    data.emitter_ts = _deserializer.uint64(buffer, bufferOffset);
    return data;
  }

  static getMessageSize(object) {
    return 8;
  }

  static datatype() {
    // Returns string type for a message object
    return 'eval_interfaces/Evaluation';
  }

  static md5sum() {
    //Returns md5sum for a message object
    return '476ebe5b5ffd3d3a307606dab9b18299';
  }

  static messageDefinition() {
    // Returns full string definition for message
    return `
    uint64 emitter_ts
    
    `;
  }

  static Resolve(msg) {
    // deep-construct a valid message object instance of whatever was passed in
    if (typeof msg !== 'object' || msg === null) {
      msg = {};
    }
    const resolved = new Evaluation(null);
    if (msg.emitter_ts !== undefined) {
      resolved.emitter_ts = msg.emitter_ts;
    }
    else {
      resolved.emitter_ts = 0
    }

    return resolved;
    }
};

module.exports = Evaluation;
