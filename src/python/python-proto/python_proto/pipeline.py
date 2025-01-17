from __future__ import annotations

import dataclasses
import datetime
import uuid
from typing import Generic

from google.protobuf.any_pb2 import Any as _Any
from graplinc.grapl.pipeline.v1beta1.types_pb2 import Envelope as _Envelope
from graplinc.grapl.pipeline.v1beta1.types_pb2 import RawLog as _RawLog
from python_proto.common import Timestamp, Uuid
from python_proto.serde import I, SerDe, SerDeWithInner


@dataclasses.dataclass(frozen=True)
class Envelope(SerDeWithInner[I, _Envelope], Generic[I]):
    trace_id: uuid.UUID
    tenant_id: uuid.UUID
    event_source_id: uuid.UUID
    retry_count: int
    created_time: datetime.datetime
    last_updated_time: datetime.datetime

    inner_message: I
    _proto_cls = _Envelope

    @classmethod
    def from_proto(
        cls: type[Envelope[I]], proto_envelope: _Envelope, inner_cls: type[I]
    ) -> Envelope[I]:
        inner_message_proto = inner_cls.new_proto()
        proto_envelope.inner_message.Unpack(inner_message_proto)
        inner_message = inner_cls.from_proto(inner_message_proto)
        return cls(
            trace_id=Uuid.from_proto(proto_envelope.trace_id).into_uuid(),
            tenant_id=Uuid.from_proto(proto_envelope.tenant_id).into_uuid(),
            event_source_id=Uuid.from_proto(proto_envelope.event_source_id).into_uuid(),
            retry_count=proto_envelope.retry_count,
            created_time=Timestamp.from_proto(
                proto_envelope.created_time
            ).into_datetime(),
            last_updated_time=Timestamp.from_proto(
                proto_envelope.last_updated_time
            ).into_datetime(),
            inner_message=inner_message,
        )

    def into_proto(self) -> _Envelope:
        proto_envelope = _Envelope()
        proto_envelope.trace_id.CopyFrom(Uuid.from_uuid(self.trace_id).into_proto())
        proto_envelope.tenant_id.CopyFrom(Uuid.from_uuid(self.tenant_id).into_proto())
        proto_envelope.event_source_id.CopyFrom(
            Uuid.from_uuid(self.event_source_id).into_proto()
        )
        proto_envelope.retry_count = self.retry_count
        proto_envelope.created_time.CopyFrom(
            Timestamp.from_datetime(self.created_time).into_proto()
        )
        proto_envelope.last_updated_time.CopyFrom(
            Timestamp.from_datetime(self.last_updated_time).into_proto()
        )
        inner_message = _Any()
        inner_message.Pack(
            self.inner_message.into_proto(),
            type_url_prefix=b"graplsecurity.com",
        )
        proto_envelope.inner_message.CopyFrom(inner_message)
        return proto_envelope


@dataclasses.dataclass(frozen=True)
class RawLog(SerDe[_RawLog]):
    log_event: bytes
    _proto_cls = _RawLog

    @classmethod
    def from_proto(cls, proto_raw_log: _RawLog) -> RawLog:
        return RawLog(log_event=proto_raw_log.log_event)

    def into_proto(self) -> _RawLog:
        proto_raw_log = _RawLog()
        proto_raw_log.log_event = self.log_event
        return proto_raw_log
