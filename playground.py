from drivers.python.driver_test import NobreakServer
from clients.python.client import NobreakConnection, NobreakClient

nobreak_binary_path = "/home/jacques/Documents/nobreak/server/target/debug/nobreak_server"

with NobreakServer(nobreak_binary_path) as server:
    connection = NobreakConnection(server.api_url)
    client = NobreakClient(connection)

    client.log("QWE", b"asd")
    client.log("lala", b"test")

    if sub_client := client.sub("D"):
        sub_client.log("E", b"qwe")
