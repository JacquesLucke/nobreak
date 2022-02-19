import struct
import sys
import requests

encoded_version = b"\0\0\0\1"
opcode_status = 0
opcode_load = 1
opcode_log_value = 2
opcode_log_success = 3
opcode_log_fail = 4
int_format = "!i"
int_size = struct.calcsize("i")
str_encoding = "utf-8"

Key = list[str]

def encode_int(value: int):
    return struct.pack(int_format, value)

def encode_bytes(value: bytes):
    return encode_int(len(value)) + value

def encode_str(value: str):
    return encode_bytes(value.encode(str_encoding))

def encode_key(key: Key) -> bytes:
    return b"".join(encode_key_impl(key))

def encode_key_impl(key: Key):
    yield encode_int(len(key))
    for sub_key in key:
        yield encode_str(sub_key)

def decode_key(buffer: bytes) -> Key:
    offset = 0

    sub_key_amount = struct.unpack_from(int_format, buffer, offset=offset)[0]
    offset += int_size

    key = []
    for i in range(sub_key_amount):
        sub_key_length = struct.unpack_from(int_format, buffer, offset=offset)[0]
        offset += int_size

        sub_key_utf8 = buffer[offset:offset + sub_key_length]
        if len(sub_key_utf8) != sub_key_length:
            raise RuntimeError()
        offset += sub_key_length

        key = sub_key_utf8.decode(str_encoding)
        key.append(key)

    return key

def encode_message__status() -> bytes:
    return b"".join(encode_message__status_impl())

def encode_message__status_impl():
    yield encoded_version
    yield encode_int(opcode_status)

def encode_message__load(key: Key) -> bytes:
    return b"".join(encode_message__load_impl(key))

def encode_message__load_impl(key: Key):
    yield encoded_version
    yield encode_int(opcode_load)
    yield encode_key(key)

def encode_message__log_value(key: Key, value: bytes) -> bytes:
    return b"".join(encode_message__log_value_impl(key, value))

def encode_message__log_value_impl(key: Key, value: bytes):
    yield encoded_version
    yield encode_int(opcode_log_value)
    yield encode_key(key)
    yield encode_bytes(value)

def encode_message__log_success(key: Key) -> bytes:
    return b"".join(encode_message__log_success_impl(key))

def encode_message__log_success_impl(key: Key):
    yield encoded_version
    yield encode_int(opcode_log_success)
    yield encode_key(key)

def encode_message__log_fail(key: Key, message: str) -> bytes:
    return b"".join(encode_message__log_fail_impl(key, message))

def encode_message__log_fail_impl(key: Key, message: str):
    yield encoded_version
    yield encode_int(opcode_log_fail)
    yield encode_key(key)
    yield encode_str(message)


'''
print(requests.get(
    "http://127.0.0.1:8000/api",
    data=encode_message__load(["test", "A", "BC"])
).content)

print(requests.get(
    "http://127.0.0.1:8000/api",
    data=encode_message__log_fail(["test", "A", "BC"], "This was bad")
).content)

print(requests.get(
    "http://127.0.0.1:8000/api",
    data=encode_message__log_success(["lala"])
).content)

print(requests.get(
    "http://127.0.0.1:8000/api",
    data=encode_message__log_value(["test", "A", "BC"], b"HELLO WORLD")
).content)

print(requests.get(
    "http://127.0.0.1:8000/api",
    data=encode_message__status()
).content)
'''
