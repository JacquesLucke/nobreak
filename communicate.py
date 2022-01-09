import struct
import sys

int_format = "!i"
int_size = struct.calcsize("i")
str_encoding = "utf-8"

def encode_full_key(full_key: list[str]):
    parts = []
    parts.append(struct.pack(int_format, len(full_key)))
    for key in full_key:
        key_utf8 = key.encode(str_encoding)
        parts.append(struct.pack(int_format, len(key_utf8)))
        parts.append(key_utf8)
    return b"".join(parts)

def decode_full_key(buffer: bytes):
    offset = 0

    key_amount = struct.unpack_from(int_format, buffer, offset=offset)[0]
    offset += int_size

    full_key = []
    for i in range(key_amount):
        key_length = struct.unpack_from(int_format, buffer, offset=offset)[0]
        offset += int_size

        key_utf8 = buffer[offset:offset + key_length]
        if len(key_utf8) != key_length:
            raise RuntimeError()
        offset += key_length

        key = key_utf8.decode(str_encoding)
        full_key.append(key)

    return full_key



encoded = encode_full_key(["test", "a", "b", "c"])
for b in encoded:
    print(b, end=" ")
# decoded = decode_full_key(encoded)
# print(encoded)
# print(decoded)

print()
