from __future__ import annotations

import requests
from pprint import pprint
import enum
from . communicate import (
    encode_message__status,
    encode_message__load,
    encode_message__log_value,
    encode_message__log_success,
    encode_message__log_fail,
)

class NobreakOperationMode(enum.Enum):
    UPDATE = enum.auto()
    CHECK = enum.auto()

class NobreakConnection:
    def __init__(self, server_url: str):
        self._server_url = server_url

        response = requests.get(server_url, data=encode_message__status())
        if response.status_code != 200:
            raise ConnectionError("Could not connect to nobreak server.")
        self.operation_mode = NobreakOperationMode.UPDATE

    def log(self, key: list[str], value: bytes):
        requests.get(self._server_url, data=encode_message__log_value(key, value))

    def get(self, key: list[str]) -> bytes | None:
        response = requests.get(self._server_url, data=encode_message__load(key))
        return "test"

    def fail(self, key: list[str], msg: str):
        requests.get(self._server_url, data=encode_message__log_fail(key, msg))

class NobreakClient:
    def __init__(self, connection: NobreakConnection, parent_key: list[str] | None = None):
        self.connection = connection
        self.parent_key = [] if parent_key is None else parent_key

    def log(self, sub_key: str, value: bytes):
        key = self.parent_key + [sub_key]
        if self.connection.operation_mode == NobreakOperationMode.UPDATE:
            self.connection.log(key, value)
        elif self.connection.operation_mode == NobreakOperationMode.CHECK:
            stored_value = self.connection.get(key)
            if stored_value is None:
                print("Value was not stored")
            elif value == stored_value:
                print("Equal:", sub_key, value)
            else:
                print("Not Equal:", sub_key, value, stored_value)
                self.connection.fail(key, f"{value} != {stored_value}")
        else:
            raise RuntimeError("Unknown nobreak operation mode.")

    def sub(self, sub_key: str) -> NobreakClient | None:
        return NobreakClient(self.connection, self.parent_key + [sub_key])
