// Generated by gencpp from file eval_interfaces/Evaluation.msg
// DO NOT EDIT!


#ifndef EVAL_INTERFACES_MESSAGE_EVALUATION_H
#define EVAL_INTERFACES_MESSAGE_EVALUATION_H


#include <string>
#include <vector>
#include <map>

#include <ros/types.h>
#include <ros/serialization.h>
#include <ros/builtin_message_traits.h>
#include <ros/message_operations.h>


namespace eval_interfaces
{
template <class ContainerAllocator>
struct Evaluation_
{
  typedef Evaluation_<ContainerAllocator> Type;

  Evaluation_()
    : emitter_ts(0)  {
    }
  Evaluation_(const ContainerAllocator& _alloc)
    : emitter_ts(0)  {
  (void)_alloc;
    }



   typedef uint64_t _emitter_ts_type;
  _emitter_ts_type emitter_ts;





  typedef boost::shared_ptr< ::eval_interfaces::Evaluation_<ContainerAllocator> > Ptr;
  typedef boost::shared_ptr< ::eval_interfaces::Evaluation_<ContainerAllocator> const> ConstPtr;

}; // struct Evaluation_

typedef ::eval_interfaces::Evaluation_<std::allocator<void> > Evaluation;

typedef boost::shared_ptr< ::eval_interfaces::Evaluation > EvaluationPtr;
typedef boost::shared_ptr< ::eval_interfaces::Evaluation const> EvaluationConstPtr;

// constants requiring out of line definition



template<typename ContainerAllocator>
std::ostream& operator<<(std::ostream& s, const ::eval_interfaces::Evaluation_<ContainerAllocator> & v)
{
ros::message_operations::Printer< ::eval_interfaces::Evaluation_<ContainerAllocator> >::stream(s, "", v);
return s;
}


template<typename ContainerAllocator1, typename ContainerAllocator2>
bool operator==(const ::eval_interfaces::Evaluation_<ContainerAllocator1> & lhs, const ::eval_interfaces::Evaluation_<ContainerAllocator2> & rhs)
{
  return lhs.emitter_ts == rhs.emitter_ts;
}

template<typename ContainerAllocator1, typename ContainerAllocator2>
bool operator!=(const ::eval_interfaces::Evaluation_<ContainerAllocator1> & lhs, const ::eval_interfaces::Evaluation_<ContainerAllocator2> & rhs)
{
  return !(lhs == rhs);
}


} // namespace eval_interfaces

namespace ros
{
namespace message_traits
{





template <class ContainerAllocator>
struct IsMessage< ::eval_interfaces::Evaluation_<ContainerAllocator> >
  : TrueType
  { };

template <class ContainerAllocator>
struct IsMessage< ::eval_interfaces::Evaluation_<ContainerAllocator> const>
  : TrueType
  { };

template <class ContainerAllocator>
struct IsFixedSize< ::eval_interfaces::Evaluation_<ContainerAllocator> >
  : TrueType
  { };

template <class ContainerAllocator>
struct IsFixedSize< ::eval_interfaces::Evaluation_<ContainerAllocator> const>
  : TrueType
  { };

template <class ContainerAllocator>
struct HasHeader< ::eval_interfaces::Evaluation_<ContainerAllocator> >
  : FalseType
  { };

template <class ContainerAllocator>
struct HasHeader< ::eval_interfaces::Evaluation_<ContainerAllocator> const>
  : FalseType
  { };


template<class ContainerAllocator>
struct MD5Sum< ::eval_interfaces::Evaluation_<ContainerAllocator> >
{
  static const char* value()
  {
    return "476ebe5b5ffd3d3a307606dab9b18299";
  }

  static const char* value(const ::eval_interfaces::Evaluation_<ContainerAllocator>&) { return value(); }
  static const uint64_t static_value1 = 0x476ebe5b5ffd3d3aULL;
  static const uint64_t static_value2 = 0x307606dab9b18299ULL;
};

template<class ContainerAllocator>
struct DataType< ::eval_interfaces::Evaluation_<ContainerAllocator> >
{
  static const char* value()
  {
    return "eval_interfaces/Evaluation";
  }

  static const char* value(const ::eval_interfaces::Evaluation_<ContainerAllocator>&) { return value(); }
};

template<class ContainerAllocator>
struct Definition< ::eval_interfaces::Evaluation_<ContainerAllocator> >
{
  static const char* value()
  {
    return "uint64 emitter_ts\n"
;
  }

  static const char* value(const ::eval_interfaces::Evaluation_<ContainerAllocator>&) { return value(); }
};

} // namespace message_traits
} // namespace ros

namespace ros
{
namespace serialization
{

  template<class ContainerAllocator> struct Serializer< ::eval_interfaces::Evaluation_<ContainerAllocator> >
  {
    template<typename Stream, typename T> inline static void allInOne(Stream& stream, T m)
    {
      stream.next(m.emitter_ts);
    }

    ROS_DECLARE_ALLINONE_SERIALIZER
  }; // struct Evaluation_

} // namespace serialization
} // namespace ros

namespace ros
{
namespace message_operations
{

template<class ContainerAllocator>
struct Printer< ::eval_interfaces::Evaluation_<ContainerAllocator> >
{
  template<typename Stream> static void stream(Stream& s, const std::string& indent, const ::eval_interfaces::Evaluation_<ContainerAllocator>& v)
  {
    s << indent << "emitter_ts: ";
    Printer<uint64_t>::stream(s, indent + "  ", v.emitter_ts);
  }
};

} // namespace message_operations
} // namespace ros

#endif // EVAL_INTERFACES_MESSAGE_EVALUATION_H
