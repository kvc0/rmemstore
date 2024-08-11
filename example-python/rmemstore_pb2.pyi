from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Get(_message.Message):
    __slots__ = ["key"]
    KEY_FIELD_NUMBER: _ClassVar[int]
    key: bytes
    def __init__(self, key: _Optional[bytes] = ...) -> None: ...

class Map(_message.Message):
    __slots__ = ["map"]
    class MapEntry(_message.Message):
        __slots__ = ["key", "value"]
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: Value
        def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[Value, _Mapping]] = ...) -> None: ...
    MAP_FIELD_NUMBER: _ClassVar[int]
    map: _containers.MessageMap[str, Value]
    def __init__(self, map: _Optional[_Mapping[str, Value]] = ...) -> None: ...

class Put(_message.Message):
    __slots__ = ["key", "value"]
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: bytes
    value: Value
    def __init__(self, key: _Optional[bytes] = ..., value: _Optional[_Union[Value, _Mapping]] = ...) -> None: ...

class Response(_message.Message):
    __slots__ = ["id", "ok", "value"]
    ID_FIELD_NUMBER: _ClassVar[int]
    OK_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    id: int
    ok: bool
    value: Value
    def __init__(self, id: _Optional[int] = ..., ok: bool = ..., value: _Optional[_Union[Value, _Mapping]] = ...) -> None: ...

class Rpc(_message.Message):
    __slots__ = ["get", "id", "put"]
    GET_FIELD_NUMBER: _ClassVar[int]
    ID_FIELD_NUMBER: _ClassVar[int]
    PUT_FIELD_NUMBER: _ClassVar[int]
    get: Get
    id: int
    put: Put
    def __init__(self, id: _Optional[int] = ..., put: _Optional[_Union[Put, _Mapping]] = ..., get: _Optional[_Union[Get, _Mapping]] = ...) -> None: ...

class Value(_message.Message):
    __slots__ = ["blob", "map", "string"]
    BLOB_FIELD_NUMBER: _ClassVar[int]
    MAP_FIELD_NUMBER: _ClassVar[int]
    STRING_FIELD_NUMBER: _ClassVar[int]
    blob: bytes
    map: Map
    string: str
    def __init__(self, blob: _Optional[bytes] = ..., string: _Optional[str] = ..., map: _Optional[_Union[Map, _Mapping]] = ...) -> None: ...
