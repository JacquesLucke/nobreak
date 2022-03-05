from __future__ import annotations
from .client import NobreakClient, NobreakOperationMode


class NobreakTester:
    def __init__(self, client: NobreakClient, parent_key: list[str] | None = None):
        self.client = client
        self.parent_key = [] if parent_key is None else parent_key

    def test(self, sub_key: str, value: bytes):
        key = self.parent_key + [sub_key]
        if self.client.operation_mode == NobreakOperationMode.UPDATE:
            self.client.log(key, value)
        elif self.client.operation_mode == NobreakOperationMode.CHECK:
            stored_value = self.client.get(key)
            if stored_value is None:
                print("Value was not stored")
            elif value == stored_value:
                print("Equal:", sub_key, value)
            else:
                print("Not Equal:", sub_key, value, stored_value)
                self.client.fail(key, f"{value} != {stored_value}")
        else:
            raise RuntimeError("Unknown nobreak operation mode.")

    def sub(self, sub_key: str) -> NobreakTester | None:
        return NobreakTester(self.client, self.parent_key + [sub_key])
