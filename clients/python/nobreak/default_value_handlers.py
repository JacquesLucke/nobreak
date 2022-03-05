from __future__ import annotations
import struct
from .value_handler import (
    CompareResult,
    SuccessCompareResult,
    FailCompareResult,
    ValueHandler,
)
import textwrap

default_value_handler_by_type = {}


def register_default_value_handler(
    value_handler: ValueHandler, supported_types: type | list[type]
):
    if type(supported_types) == type:
        supported_types = [supported_types]

    for supported_type in supported_types:
        default_value_handler_by_type[supported_type] = value_handler


def get_default_value_handler(data_type: type):
    return default_value_handler_by_type.get(data_type)


class IntValueHandler(ValueHandler):
    def serialize(self, value: int) -> bytes:
        return value.to_bytes(
            length=(8 + (value + (value < 0)).bit_length()) // 8,
            byteorder="big",
            signed=True,
        )

    def deserialize(self, data: bytes) -> int:
        return int.from_bytes(data, byteorder="big", signed=True)

    def compare(self, a: int, b_bytes: bytes) -> CompareResult:
        b = self.deserialize(b_bytes)
        if a == b:
            return SuccessCompareResult()
        return FailCompareResult(f"values are not equal: {a} and {b}")


class FloatValueHandler(ValueHandler):
    def __init__(self, absolute_epsilon: float = 0.0001):
        self.absolute_epsilon = absolute_epsilon

    def serialize(self, value: float) -> bytes:
        return struct.pack(">d", value)

    def deserialize(self, data: bytes) -> float:
        return struct.unpack(">d", data)[0]

    def compare(self, a: float, b_bytes: bytes) -> CompareResult:
        b = self.deserialize(b_bytes)
        diff = abs(a - b)
        if diff <= self.absolute_epsilon:
            return SuccessCompareResult()
        return FailCompareResult(
            f"value are not equal: {a} and {b} (difference: {diff}, epsilon: {self.absolute_epsilon}"
        )


class StringValueHandler(ValueHandler):
    def serialize(self, value: str) -> bytes:
        return value.encode("utf-8")

    def deserialize(self, data: bytes) -> str:
        return data.decode("utf-8")

    def compare(self, a: float, b_bytes: bytes) -> CompareResult:
        b = self.deserialize(b_bytes)
        if a == b:
            return SuccessCompareResult()
        return FailCompareResult(
            textwrap.dedent(
                f"""\
                Strings are not equal:
                {repr(a)}
                {repr(b)}
                """
            )
        )


class BytesValueHandler(ValueHandler):
    def serialize(self, value: bytes) -> bytes:
        return value

    def compare(self, a: bytes, b: bytes) -> CompareResult:
        if a == b:
            return SuccessCompareResult()
        return FailCompareResult(
            textwrap.dedent(
                f"""\
                Bytes are not equal:
                {repr(a)}
                {repr(b)}
                """
            )
        )


register_default_value_handler(IntValueHandler(), int)
register_default_value_handler(FloatValueHandler(), float)
register_default_value_handler(StringValueHandler(), str)
register_default_value_handler(BytesValueHandler(), bytes)
