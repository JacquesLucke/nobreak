from __future__ import annotations

import requests
from pprint import pprint
import enum
from .communicate import (
    encode_message__status,
    encode_message__load,
    encode_message__log_value,
    encode_message__log_success,
    encode_message__log_fail,
)


class OperationMode(enum.Enum):
    UPDATE = enum.auto()
    CHECK = enum.auto()


class Client:
    def __init__(self, server_url: str, mode: str | OperationMode):
        response = requests.get(server_url, data=encode_message__status())
        if response.status_code != 200:
            raise ConnectionError("Could not connect to nobreak server.")

        self._server_url = server_url

        if isinstance(mode, str):
            mode = OperationMode[mode]
        self._mode = mode

    def log(self, key: list[str], value: bytes):
        requests.get(self._server_url, data=encode_message__log_value(key, value))

    def get(self, key: list[str]) -> bytes | None:
        response = requests.get(self._server_url, data=encode_message__load(key))
        success = response.content[0]
        if success:
            return response.content[1:]
        return None

    def fail(self, key: list[str], msg: str):
        requests.get(self._server_url, data=encode_message__log_fail(key, msg))
