//
// Copyright (c) 2022 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//

use std::num::NonZeroUsize;
use zenoh_buffers::reader::BacktrackableReader;
use zenoh_buffers::reader::DidntRead;
use zenoh_buffers::reader::Reader;
use zenoh_codec::{
    RCodec, Zenoh060, Zenoh060Header, Zenoh060HeaderReplyContext, Zenoh060Reliability,
};
use zenoh_protocol::common::{imsg, Attachment};
use zenoh_protocol::core::Channel;
use zenoh_protocol::core::Priority;
use zenoh_protocol::transport::{
    tmsg, Close, FrameHeader, FrameKind, InitAck, InitSyn, Join, KeepAlive, OpenAck, OpenSyn,
};
use zenoh_protocol::zenoh::Data;
use zenoh_protocol::zenoh::Declare;
use zenoh_protocol::zenoh::LinkStateList;
use zenoh_protocol::zenoh::Pull;
use zenoh_protocol::zenoh::Query;
use zenoh_protocol::zenoh::Unit;
use zenoh_protocol::zenoh::{zmsg, ReplyContext, RoutingContext};

use crate::types::SizedData;
use crate::types::SizedDeclare;
use crate::types::SizedLinkStateList;
use crate::types::SizedPull;
use crate::types::SizedQuery;
use crate::types::SizedTransportMessage;
use crate::types::SizedUnit;
use crate::types::SizedZenohBody;
use crate::types::{
    SizedClose, SizedFrame, SizedFramePayload, SizedInitAck, SizedInitSyn, SizedJoin,
    SizedKeepAlive, SizedOpenAck, SizedOpenSyn, SizedTransportBody, SizedZenohMessage,
};

impl<R> RCodec<SizedTransportMessage, &mut R> for Zenoh060
where
    R: Reader + BacktrackableReader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<SizedTransportMessage, Self::Error> {
        let len = reader.remaining();

        let mut codec = Zenoh060Header {
            header: self.read(&mut *reader)?,
            ..Default::default()
        };
        let mut attachment: Option<Attachment> = None;
        if imsg::mid(codec.header) == tmsg::id::ATTACHMENT {
            let a: Attachment = codec.read(&mut *reader)?;
            attachment = Some(a);
            codec.header = self.read(&mut *reader)?;
        }
        let body = match imsg::mid(codec.header) {
            tmsg::id::INIT => {
                if !imsg::has_flag(codec.header, tmsg::flag::A) {
                    SizedTransportBody::InitSyn(codec.read(&mut *reader)?)
                } else {
                    SizedTransportBody::InitAck(codec.read(&mut *reader)?)
                }
            }
            tmsg::id::OPEN => {
                if !imsg::has_flag(codec.header, tmsg::flag::A) {
                    SizedTransportBody::OpenSyn(codec.read(&mut *reader)?)
                } else {
                    SizedTransportBody::OpenAck(codec.read(&mut *reader)?)
                }
            }
            tmsg::id::JOIN => SizedTransportBody::Join(codec.read(&mut *reader)?),
            tmsg::id::CLOSE => SizedTransportBody::Close(codec.read(&mut *reader)?),
            tmsg::id::KEEP_ALIVE => SizedTransportBody::KeepAlive(codec.read(&mut *reader)?),
            tmsg::id::PRIORITY | tmsg::id::FRAME => {
                SizedTransportBody::Frame(codec.read(&mut *reader)?)
            }
            _ => return Err(DidntRead),
        };

        let size = NonZeroUsize::new(len - reader.remaining()).ok_or_else(|| DidntRead)?;
        Ok(SizedTransportMessage {
            body,
            attachment,
            size,
        })
    }
}

impl<R> RCodec<SizedInitSyn, &mut R> for Zenoh060
where
    R: Reader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<SizedInitSyn, Self::Error> {
        let len = reader.remaining();
        let msg: InitSyn = self.read(&mut *reader)?;
        let size = NonZeroUsize::new(len - reader.remaining() + 1).ok_or_else(|| DidntRead)?;
        Ok(SizedInitSyn(size, msg))
    }
}

impl<R> RCodec<SizedInitSyn, &mut R> for Zenoh060Header
where
    R: Reader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<SizedInitSyn, Self::Error> {
        let len = reader.remaining();
        let msg: InitSyn = self.read(&mut *reader)?;
        let size = NonZeroUsize::new(len - reader.remaining() + 1).ok_or_else(|| DidntRead)?;
        Ok(SizedInitSyn(size, msg))
    }
}

impl<R> RCodec<SizedInitAck, &mut R> for Zenoh060
where
    R: Reader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<SizedInitAck, Self::Error> {
        let len = reader.remaining();
        let msg: InitAck = self.read(&mut *reader)?;
        let size = NonZeroUsize::new(len - reader.remaining() + 1).ok_or_else(|| DidntRead)?;
        Ok(SizedInitAck(size, msg))
    }
}

impl<R> RCodec<SizedInitAck, &mut R> for Zenoh060Header
where
    R: Reader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<SizedInitAck, Self::Error> {
        let len = reader.remaining();
        let msg: InitAck = self.read(&mut *reader)?;
        let size = NonZeroUsize::new(len - reader.remaining() + 1).ok_or_else(|| DidntRead)?;
        Ok(SizedInitAck(size, msg))
    }
}

impl<R> RCodec<SizedOpenSyn, &mut R> for Zenoh060
where
    R: Reader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<SizedOpenSyn, Self::Error> {
        let len = reader.remaining();
        let msg: OpenSyn = self.read(&mut *reader)?;
        let size = NonZeroUsize::new(len - reader.remaining() + 1).ok_or_else(|| DidntRead)?;
        Ok(SizedOpenSyn(size, msg))
    }
}

impl<R> RCodec<SizedOpenSyn, &mut R> for Zenoh060Header
where
    R: Reader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<SizedOpenSyn, Self::Error> {
        let len = reader.remaining();
        let msg: OpenSyn = self.read(&mut *reader)?;
        let size = NonZeroUsize::new(len - reader.remaining() + 1).ok_or_else(|| DidntRead)?;
        Ok(SizedOpenSyn(size, msg))
    }
}

impl<R> RCodec<SizedOpenAck, &mut R> for Zenoh060
where
    R: Reader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<SizedOpenAck, Self::Error> {
        let len = reader.remaining();
        let msg: OpenAck = self.read(&mut *reader)?;
        let size = NonZeroUsize::new(len - reader.remaining() + 1).ok_or_else(|| DidntRead)?;
        Ok(SizedOpenAck(size, msg))
    }
}

impl<R> RCodec<SizedOpenAck, &mut R> for Zenoh060Header
where
    R: Reader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<SizedOpenAck, Self::Error> {
        let len = reader.remaining();
        let msg: OpenAck = self.read(&mut *reader)?;
        let size = NonZeroUsize::new(len - reader.remaining() + 1).ok_or_else(|| DidntRead)?;
        Ok(SizedOpenAck(size, msg))
    }
}

impl<R> RCodec<SizedJoin, &mut R> for Zenoh060
where
    R: Reader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<SizedJoin, Self::Error> {
        let len = reader.remaining();
        let msg: Join = self.read(&mut *reader)?;
        let size = NonZeroUsize::new(len - reader.remaining() + 1).ok_or_else(|| DidntRead)?;
        Ok(SizedJoin(size, msg))
    }
}

impl<R> RCodec<SizedJoin, &mut R> for Zenoh060Header
where
    R: Reader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<SizedJoin, Self::Error> {
        let len = reader.remaining();
        let msg: Join = self.read(&mut *reader)?;
        let size = NonZeroUsize::new(len - reader.remaining() + 1).ok_or_else(|| DidntRead)?;
        Ok(SizedJoin(size, msg))
    }
}

impl<R> RCodec<SizedClose, &mut R> for Zenoh060
where
    R: Reader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<SizedClose, Self::Error> {
        let len = reader.remaining();
        let msg: Close = self.read(&mut *reader)?;
        let size = NonZeroUsize::new(len - reader.remaining() + 1).ok_or_else(|| DidntRead)?;
        Ok(SizedClose(size, msg))
    }
}

impl<R> RCodec<SizedClose, &mut R> for Zenoh060Header
where
    R: Reader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<SizedClose, Self::Error> {
        let len = reader.remaining();
        let msg: Close = self.read(&mut *reader)?;
        let size = NonZeroUsize::new(len - reader.remaining() + 1).ok_or_else(|| DidntRead)?;
        Ok(SizedClose(size, msg))
    }
}

impl<R> RCodec<SizedKeepAlive, &mut R> for Zenoh060
where
    R: Reader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<SizedKeepAlive, Self::Error> {
        let len = reader.remaining();
        let msg: KeepAlive = self.read(&mut *reader)?;
        let size = NonZeroUsize::new(len - reader.remaining() + 1).ok_or_else(|| DidntRead)?;
        Ok(SizedKeepAlive(size, msg))
    }
}

impl<R> RCodec<SizedKeepAlive, &mut R> for Zenoh060Header
where
    R: Reader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<SizedKeepAlive, Self::Error> {
        let len = reader.remaining();
        let msg: KeepAlive = self.read(&mut *reader)?;
        let size = NonZeroUsize::new(len - reader.remaining() + 1).ok_or_else(|| DidntRead)?;
        Ok(SizedKeepAlive(size, msg))
    }
}

impl<R> RCodec<SizedFrame, &mut R> for Zenoh060Header
where
    R: Reader + BacktrackableReader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<SizedFrame, Self::Error> {
        let len = reader.remaining();
        let header: FrameHeader = self.read(&mut *reader)?;

        let payload = match header.kind {
            FrameKind::Messages => {
                let rcode = Zenoh060Reliability {
                    reliability: header.channel.reliability,
                    ..Default::default()
                };

                let mut messages: Vec<SizedZenohMessage> = Vec::with_capacity(1);
                while reader.can_read() {
                    let mark = reader.mark();
                    let res: Result<SizedZenohMessage, DidntRead> = rcode.read(&mut *reader);
                    match res {
                        Ok(m) => messages.push(m),
                        Err(_) => {
                            reader.rewind(mark);
                            break;
                        }
                    }
                }
                SizedFramePayload::Messages { messages }
            }
            FrameKind::SomeFragment | FrameKind::LastFragment => {
                // A fragmented frame is not supposed to be followed by
                // any other frame in the same batch. Read all the bytes.
                let buffer = reader.read_zslice(reader.remaining())?;
                let is_final = header.kind == FrameKind::LastFragment;
                SizedFramePayload::Fragment { buffer, is_final }
            }
        };

        let size = NonZeroUsize::new(len - reader.remaining()).ok_or_else(|| DidntRead)?;
        Ok(SizedFrame {
            channel: header.channel,
            sn: header.sn,
            payload,
            size,
        })
    }
}

impl<R> RCodec<SizedZenohMessage, &mut R> for Zenoh060Reliability
where
    R: Reader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<SizedZenohMessage, Self::Error> {
        let mut codec = Zenoh060Header {
            header: self.codec.read(&mut *reader)?,
            ..Default::default()
        };

        let attachment = if imsg::mid(codec.header) == imsg::id::ATTACHMENT {
            let a: Attachment = codec.read(&mut *reader)?;
            codec.header = self.codec.read(&mut *reader)?;
            Some(a)
        } else {
            None
        };
        let routing_context = if imsg::mid(codec.header) == zmsg::id::ROUTING_CONTEXT {
            let r: RoutingContext = codec.read(&mut *reader)?;
            codec.header = self.codec.read(&mut *reader)?;
            Some(r)
        } else {
            None
        };
        let priority = if imsg::mid(codec.header) == zmsg::id::PRIORITY {
            let p: Priority = codec.read(&mut *reader)?;
            codec.header = self.codec.read(&mut *reader)?;
            p
        } else {
            Priority::default()
        };

        let body = match imsg::mid(codec.header) {
            zmsg::id::REPLY_CONTEXT => {
                let rc: ReplyContext = codec.read(&mut *reader)?;
                let rodec = Zenoh060HeaderReplyContext {
                    header: self.codec.read(&mut *reader)?,
                    reply_context: Some(rc),
                    ..Default::default()
                };
                match imsg::mid(rodec.header) {
                    zmsg::id::DATA => SizedZenohBody::Data(rodec.read(&mut *reader)?),
                    zmsg::id::UNIT => SizedZenohBody::Unit(rodec.read(&mut *reader)?),
                    _ => return Err(DidntRead),
                }
            }
            zmsg::id::DATA => {
                let rodec = Zenoh060HeaderReplyContext {
                    header: codec.header,
                    ..Default::default()
                };
                SizedZenohBody::Data(rodec.read(&mut *reader)?)
            }
            zmsg::id::UNIT => {
                let rodec = Zenoh060HeaderReplyContext {
                    header: codec.header,
                    ..Default::default()
                };
                SizedZenohBody::Unit(rodec.read(&mut *reader)?)
            }
            zmsg::id::PULL => SizedZenohBody::Pull(codec.read(&mut *reader)?),
            zmsg::id::QUERY => SizedZenohBody::Query(codec.read(&mut *reader)?),
            zmsg::id::DECLARE => SizedZenohBody::Declare(codec.read(&mut *reader)?),
            zmsg::id::LINK_STATE_LIST => SizedZenohBody::LinkStateList(codec.read(&mut *reader)?),
            _ => return Err(DidntRead),
        };

        Ok(SizedZenohMessage {
            body,
            attachment,
            channel: Channel {
                priority,
                reliability: self.reliability,
            },
            routing_context,
        })
    }
}

impl<R> RCodec<SizedData, &mut R> for Zenoh060
where
    R: Reader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<SizedData, Self::Error> {
        let len = reader.remaining();
        let msg: Data = self.read(&mut *reader)?;
        let size = NonZeroUsize::new(len - reader.remaining() + 1).ok_or_else(|| DidntRead)?;
        Ok(SizedData(size, msg))
    }
}

impl<R> RCodec<SizedData, &mut R> for Zenoh060HeaderReplyContext
where
    R: Reader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<SizedData, Self::Error> {
        let len = reader.remaining();
        let msg: Data = self.read(&mut *reader)?;
        let size = NonZeroUsize::new(len - reader.remaining() + 1).ok_or_else(|| DidntRead)?;

        Ok(SizedData(size, msg))
    }
}

impl<R> RCodec<SizedUnit, &mut R> for Zenoh060
where
    R: Reader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<SizedUnit, Self::Error> {
        let len = reader.remaining();
        let msg: Unit = self.read(&mut *reader)?;
        let size = NonZeroUsize::new(len - reader.remaining() + 1).ok_or_else(|| DidntRead)?;
        Ok(SizedUnit(size, msg))
    }
}

impl<R> RCodec<SizedUnit, &mut R> for Zenoh060HeaderReplyContext
where
    R: Reader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<SizedUnit, Self::Error> {
        let len = reader.remaining();
        let msg: Unit = self.read(&mut *reader)?;
        let size = NonZeroUsize::new(len - reader.remaining() + 1).ok_or_else(|| DidntRead)?;
        Ok(SizedUnit(size, msg))
    }
}

impl<R> RCodec<SizedPull, &mut R> for Zenoh060
where
    R: Reader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<SizedPull, Self::Error> {
        let len = reader.remaining();
        let msg: Pull = self.read(&mut *reader)?;
        let size = NonZeroUsize::new(len - reader.remaining() + 1).ok_or_else(|| DidntRead)?;
        Ok(SizedPull(size, msg))
    }
}

impl<R> RCodec<SizedPull, &mut R> for Zenoh060Header
where
    R: Reader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<SizedPull, Self::Error> {
        let len = reader.remaining();
        let msg: Pull = self.read(&mut *reader)?;
        let size = NonZeroUsize::new(len - reader.remaining() + 1).ok_or_else(|| DidntRead)?;
        Ok(SizedPull(size, msg))
    }
}

impl<R> RCodec<SizedQuery, &mut R> for Zenoh060
where
    R: Reader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<SizedQuery, Self::Error> {
        let len = reader.remaining();
        let msg: Query = self.read(&mut *reader)?;
        let size = NonZeroUsize::new(len - reader.remaining() + 1).ok_or_else(|| DidntRead)?;
        Ok(SizedQuery(size, msg))
    }
}

impl<R> RCodec<SizedQuery, &mut R> for Zenoh060Header
where
    R: Reader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<SizedQuery, Self::Error> {
        let len = reader.remaining();
        let msg: Query = self.read(&mut *reader)?;
        let size = NonZeroUsize::new(len - reader.remaining() + 1).ok_or_else(|| DidntRead)?;
        Ok(SizedQuery(size, msg))
    }
}

impl<R> RCodec<SizedDeclare, &mut R> for Zenoh060
where
    R: Reader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<SizedDeclare, Self::Error> {
        let len = reader.remaining();
        let msg: Declare = self.read(&mut *reader)?;
        let size = NonZeroUsize::new(len - reader.remaining() + 1).ok_or_else(|| DidntRead)?;
        Ok(SizedDeclare(size, msg))
    }
}

impl<R> RCodec<SizedDeclare, &mut R> for Zenoh060Header
where
    R: Reader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<SizedDeclare, Self::Error> {
        let len = reader.remaining();
        let msg: Declare = self.read(&mut *reader)?;
        let size = NonZeroUsize::new(len - reader.remaining() + 1).ok_or_else(|| DidntRead)?;
        Ok(SizedDeclare(size, msg))
    }
}

impl<R> RCodec<SizedLinkStateList, &mut R> for Zenoh060
where
    R: Reader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<SizedLinkStateList, Self::Error> {
        let len = reader.remaining();
        let msg: LinkStateList = self.read(&mut *reader)?;
        let size = NonZeroUsize::new(len - reader.remaining() + 1).ok_or_else(|| DidntRead)?;
        Ok(SizedLinkStateList(size, msg))
    }
}

impl<R> RCodec<SizedLinkStateList, &mut R> for Zenoh060Header
where
    R: Reader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<SizedLinkStateList, Self::Error> {
        let len = reader.remaining();
        let msg: LinkStateList = self.read(&mut *reader)?;
        let size = NonZeroUsize::new(len - reader.remaining() + 1).ok_or_else(|| DidntRead)?;
        Ok(SizedLinkStateList(size, msg))
    }
}
