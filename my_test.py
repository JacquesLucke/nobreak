from __future__ import annotations

import sys
import os
import requests
from pprint import pprint
import urllib.parse
import enum

class NobreakOperationMode(enum.Enum):
    UPDATE = enum.auto()
    CHECK = enum.auto()
class NobreakConnection:
    def __init__(self, server_url: str):
        self._server_url = server_url

        response = requests.get(server_url)
        if response.status_code != 200:
            raise ConnectionError("Could not connect to nobreak server.")

        data = response.json()
        self.operation_mode = {
            "Update": NobreakOperationMode.UPDATE,
            "Check": NobreakOperationMode.CHECK,
        }[data["mode"]]
        self._log_url = urllib.parse.urljoin(server_url, data["log"])
        self._get_url = urllib.parse.urljoin(server_url, data["get"])

    @staticmethod
    def from_environment():
        server_url: str = os.environ["NOBREAK_SERVER_URL"]
        return NobreakConnection(server_url)

    def log(self, key: list[str], value: bytes):
        key_str = str(key)
        print(self._log_url)
        log_key_url = urllib.parse.urljoin(self._log_url, key_str)
        print(log_key_url)
        requests.post(log_key_url, data=value)

    def get(self, key: list[str]) -> bytes | None:
        key_str = str(key)
        get_key_url = urllib.parse.urljoin(self._get_url, key_str)
        return requests.get(get_key_url).content

class NobreakClient:
    def __init__(self, connection: NobreakConnection, parent_key: list[str] | None = None):
        self.connection = connection
        self.parent_key = [] if parent_key is None else parent_key

    def log(self, key: str, value: bytes):
        full_key = self.parent_key + [key]
        if self.connection.operation_mode == NobreakOperationMode.UPDATE:
            self.connection.log(full_key, value)
        elif self.connection.operation_mode == NobreakOperationMode.CHECK:
            stored_value = self.connection.get(full_key)
            if stored_value is None:
                print("Value was not stored")
            elif value == stored_value:
                print("Equal:", key, value)
            else:
                print("Not Equal:", key, value, stored_value)
        else:
            raise RuntimeError("Unknown nobreak operation mode.")

connection = NobreakConnection.from_environment()
client = NobreakClient(connection)

client.log("A", b"aa")
client.log("B", b"bb")
client.log("C", b"cc")
