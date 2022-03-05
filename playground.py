import sys
from pathlib import Path

sys.path.append(str(Path(__file__).parent / "drivers" / "python"))
sys.path.append(str(Path(__file__).parent / "clients" / "python"))


import nobreak
from nobreak_driver import NobreakServer

nobreak_binary_path = (
    "/home/jacques/Documents/nobreak/server/target/debug/nobreak_server"
)


with NobreakServer(nobreak_binary_path) as server:
    client = nobreak.Client(server.api_url, "CHECK")
    tester = nobreak.Tester(client)

    tester.test("QWE", 4)
    tester.test("lala", -6)
    tester.test("fas", b"my_bytes")
    tester.test("vxc", 4.1)

    if sub_tester := tester.sub("D"):
        sub_tester.test("E", "qwe")
