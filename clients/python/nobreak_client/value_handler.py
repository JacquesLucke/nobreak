from __future__ import annotations

from typing import Union


class CompareResult:
    pass


class SuccessCompareResult(CompareResult):
    pass


class FailCompareResult(CompareResult):
    def __init__(self, message: str):
        self.message = message


class ValueHandler:
    def serialize(self, value: object) -> bytes:
        raise NotImplementedError()

    def compare(self, a: object, b_bytes: bytes) -> CompareResult:
        raise NotImplementedError()
