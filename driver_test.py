from pathlib import Path
from os import PathLike
import subprocess
import time
from contextlib import contextmanager

class NobreakDriver:
    def __init__(self, url: str):
        self._url = url

@contextmanager
def nobreak(nobreak_binary: PathLike, *, port: int = 2345, address: str = "127.0.0.1"):
    args = []
    args.append(nobreak_binary)
    args.extend(["--port", str(port)])
    args.extend(["--address", address])
    process = subprocess.Popen(args)

    url = f"http://{address}:{port}"
    try:
        yield NobreakDriver(url)
    finally:
        process.kill()


nobreak_path = Path("/home/jacques/Documents/nobreak/nobreak/target/debug/nobreak")

with nobreak(nobreak_path, port=5000) as driver:
    print(driver)
    time.sleep(10)
