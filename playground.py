from drivers.python.driver_test import NobreakServer
from clients.python.client import NobreakClient, NobreakTester

nobreak_binary_path = "/home/jacques/Documents/nobreak/server/target/debug/nobreak_server"

with NobreakServer(nobreak_binary_path) as server:
    client = NobreakClient(server.api_url)
    tester = NobreakTester(client)

    tester.test("QWE", b"asd")
    tester.test("lala", b"test")

    if sub_tester := tester.sub("D"):
        sub_tester.test("E", b"qwe")
