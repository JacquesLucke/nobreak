from pathlib import Path
from os import PathLike
import subprocess
import time
import random
import requests


class NobreakServer:
    def __init__(
        self,
        nobreak_binary_path: PathLike,
        *,
        port: int = 2345,
        address: str = "127.0.0.1",
    ):
        self._args = []
        self._args.append(nobreak_binary_path)
        self._args.extend(["--port", str(port)])
        self._args.extend(["--address", address])
        self._server_id = random.randint(1, 1000000)
        self._args.extend(["--server-id", str(self._server_id)])
        self._url = f"http://{address}:{port}"

    def start(self):
        self._process = subprocess.Popen(self._args)
        delay_ms = 0.1
        for _ in range(10):
            try:
                response = requests.get(self._url + "/server_id")
                text = response.text
                if int(text) == self._server_id:
                    break
            except:
                pass
            delay_ms *= 2
        else:
            self._process.kill()
            self._process = None
            raise RuntimeError("could not communicate with server")

    def stop(self):
        if self._process is None:
            raise RuntimeError("server has not been started")
        self._process.kill()

    def __enter__(self):
        self.start()
        return self

    def __exit__(self, type, value, traceback):
        self.stop()

    @property
    def api_url(self):
        return self._url + "/api"
