def sys_path_hack():
    import sys
    from pathlib import Path

    sys.path.append(str(Path(__file__).parent / "drivers" / "python"))
    sys.path.append(str(Path(__file__).parent / "clients" / "python"))


sys_path_hack()


import subprocess
import nobreak
from nobreak_driver import NobreakServer

nobreak_binary_path = (
    "/home/jacques/Documents/nobreak/server/target/debug/nobreak_server"
)

blender_path = "/home/jacques/blender/build_release/bin/blender"
blend_file_path = "/home/jacques/Documents/nobreak/blend_test/cube.blend"
blend_script_path = "/home/jacques/Documents/nobreak/playground_blender.py"

with NobreakServer(nobreak_binary_path) as server:

    # subprocess.run(
    #     [
    #         blender_path,
    #         blend_file_path,
    #         "--python",
    #         blend_script_path,
    #         "-b",
    #     ],
    #     env={
    #         "NOBREAK_PYTHON_INCLUDE": "/home/jacques/Documents/nobreak/clients/python",
    #         "NOBREAK_API_URL": server.api_url,
    #         "NOBREAK_MODE": "CHECK",
    #     },
    # )

    client = nobreak.Client(server.api_url, "UPDATE")
    tester = nobreak.Tester(client)
    tester.test("QWE", 5)
    tester.test("lala", -6)
    tester.test("fas", b"my_bytes")
    tester.test("vxc", 4.1)

    if sub_tester := tester.sub("D"):
        sub_tester.test("E", "qwe")

    print(server.load_log())
