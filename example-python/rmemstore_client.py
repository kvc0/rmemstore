
import asyncio
from typing import Dict
import rmemstore_pb2
import struct

async def receiver(reader: asyncio.StreamReader, in_progress: Dict[int, asyncio.Future]):
    buffer = bytes()
    while True:
        more = await reader.read(8192)
        buffer += more
        while 4 < len(buffer):
            try:
                (length, consumed) = decode_varint(buffer)
                if len(buffer) < consumed + length:
                    # not complete yet
                    break
                response = rmemstore_pb2.Response.FromString(buffer[consumed:(consumed + length)])
                # print("l " + str(length) + " c " + str(consumed))
                # print(buffer)
                buffer = buffer[(consumed + length):]
                # print(buffer)
                waiting = in_progress[response.id]
                waiting.set_result(response)
            except Exception as e:
                # bad message
                print(e)
                return

def encode_varint(write, value):
    """
    Taken from https://github.com/protocolbuffers/protobuf/blob/main/python/google/protobuf/internal/encoder.py
    because holy smokes it seems really hard to serialize a varint via official public apis.
    """
    local_int2byte = struct.Struct('>B').pack
    bits = value & 0x7f
    value >>= 7
    while value:
        write(local_int2byte(0x80|bits))
        bits = value & 0x7f
        value >>= 7
    return write(local_int2byte(bits))

def decode_varint(buffer):
    """
    Taken from https://github.com/protocolbuffers/protobuf/blob/main/python/google/protobuf/internal/decoder.py
    because holy smokes it seems really hard to deserialize a varint via official public apis.
    """
    mask = (1 << 64) - 1
    result = 0
    shift = 0
    pos = 0
    while 1:
        b = buffer[pos]
        pos += 1
        result |= ((b & 0x7f) << shift)
        if not (b & 0x80):
            result &= mask
            result = int(result)
            return (result, pos)
        shift += 7
        if shift >= 64:
            raise 'Too many bytes when decoding varint.'

class Client:
    def __init__(self, reader: asyncio.StreamReader, writer: asyncio.StreamWriter) -> None:
        self.writer = writer
        self.id = 1
        self.in_progress: Dict[int, asyncio.Future] = dict()
        asyncio.create_task(receiver(reader, self.in_progress))
    
    def close(self):
        self.writer.close()

    async def put(self, key: bytes, value: str):
        command = rmemstore_pb2.Rpc(
            id=self.id,
            put=rmemstore_pb2.Put(
                key=key,
                value=rmemstore_pb2.Value(
                    string=value
                )
            )
        )
        return await self.rpc(command)

    async def get(self, key: bytes) -> None:
        # todo
        pass

    async def rpc(self, command: rmemstore_pb2.Rpc):
        future = asyncio.Future()
        self.in_progress[self.id] = future
        self.id += 1

        buffer = command.SerializeToString()
        encode_varint(self.writer.write, len(buffer))
        self.writer.write(buffer)
        await self.writer.drain()

        return await future

async def client() -> Client:
    reader, writer = await asyncio.open_connection('127.0.0.1', 9466)
    return Client(reader, writer)
