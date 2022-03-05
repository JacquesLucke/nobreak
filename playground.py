import sys
from pathlib import Path

sys.path.append(str(Path(__file__).parent / "drivers" / "python"))
sys.path.append(str(Path(__file__).parent / "clients" / "python"))


from nobreak_driver import NobreakServer
from nobreak_client import NobreakClient, NobreakTester

nobreak_binary_path = (
    "/home/jacques/Documents/nobreak/server/target/debug/nobreak_server"
)


with NobreakServer(nobreak_binary_path) as server:
    client = NobreakClient(server.api_url)
    tester = NobreakTester(client)

    tester.test("QWE", b"asd")
    tester.test("lala", b"test")

    if sub_tester := tester.sub("D"):
        sub_tester.test("E", b"qwe")
