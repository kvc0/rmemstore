# -*- coding: utf-8 -*-
# Generated by the protocol buffer compiler.  DO NOT EDIT!
# NO CHECKED-IN PROTOBUF GENCODE
# source: rmemstore.proto
# Protobuf Python Version: 5.28.2
"""Generated protocol buffer code."""
from google.protobuf import descriptor as _descriptor
from google.protobuf import descriptor_pool as _descriptor_pool
from google.protobuf import runtime_version as _runtime_version
from google.protobuf import symbol_database as _symbol_database
from google.protobuf.internal import builder as _builder
_runtime_version.ValidateProtobufRuntimeVersion(
    _runtime_version.Domain.PUBLIC,
    5,
    28,
    2,
    '',
    'rmemstore.proto'
)
# @@protoc_insertion_point(imports)

_sym_db = _symbol_database.Default()




DESCRIPTOR = _descriptor_pool.Default().AddSerializedFile(b'\n\x0frmemstore.proto\x12\trmemstore\"h\n\x03Rpc\x12\n\n\x02id\x18\x01 \x01(\x04\x12\x0c\n\x04\x63ode\x18\x02 \x01(\r\x12\x1d\n\x03put\x18\x03 \x01(\x0b\x32\x0e.rmemstore.PutH\x00\x12\x1d\n\x03get\x18\x04 \x01(\x0b\x32\x0e.rmemstore.GetH\x00\x42\t\n\x07\x63ommand\"]\n\x08Response\x12\n\n\x02id\x18\x01 \x01(\x04\x12\x0c\n\x04\x63ode\x18\x02 \x01(\r\x12\x0c\n\x02ok\x18\x03 \x01(\x08H\x00\x12!\n\x05value\x18\x04 \x01(\x0b\x32\x10.rmemstore.ValueH\x00\x42\x06\n\x04kind\"P\n\x05Value\x12\x0e\n\x04\x62lob\x18\x01 \x01(\x0cH\x00\x12\x10\n\x06string\x18\x02 \x01(\tH\x00\x12\x1d\n\x03map\x18\x03 \x01(\x0b\x32\x0e.rmemstore.MapH\x00\x42\x06\n\x04kind\"i\n\x03Map\x12$\n\x03map\x18\x01 \x03(\x0b\x32\x17.rmemstore.Map.MapEntry\x1a<\n\x08MapEntry\x12\x0b\n\x03key\x18\x01 \x01(\t\x12\x1f\n\x05value\x18\x02 \x01(\x0b\x32\x10.rmemstore.Value:\x02\x38\x01\"3\n\x03Put\x12\x0b\n\x03key\x18\x01 \x01(\x0c\x12\x1f\n\x05value\x18\x02 \x01(\x0b\x32\x10.rmemstore.Value\"\x12\n\x03Get\x12\x0b\n\x03key\x18\x01 \x01(\x0c\x62\x06proto3')

_globals = globals()
_builder.BuildMessageAndEnumDescriptors(DESCRIPTOR, _globals)
_builder.BuildTopDescriptorsAndMessages(DESCRIPTOR, 'rmemstore_pb2', _globals)
if not _descriptor._USE_C_DESCRIPTORS:
  DESCRIPTOR._loaded_options = None
  _globals['_MAP_MAPENTRY']._loaded_options = None
  _globals['_MAP_MAPENTRY']._serialized_options = b'8\001'
  _globals['_RPC']._serialized_start=30
  _globals['_RPC']._serialized_end=134
  _globals['_RESPONSE']._serialized_start=136
  _globals['_RESPONSE']._serialized_end=229
  _globals['_VALUE']._serialized_start=231
  _globals['_VALUE']._serialized_end=311
  _globals['_MAP']._serialized_start=313
  _globals['_MAP']._serialized_end=418
  _globals['_MAP_MAPENTRY']._serialized_start=358
  _globals['_MAP_MAPENTRY']._serialized_end=418
  _globals['_PUT']._serialized_start=420
  _globals['_PUT']._serialized_end=471
  _globals['_GET']._serialized_start=473
  _globals['_GET']._serialized_end=491
# @@protoc_insertion_point(module_scope)
