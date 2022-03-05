import sys
import os

sys.path.append(os.environ["NOBREAK_PYTHON_INCLUDE"])

import nobreak
import bpy

print(nobreak)

client = nobreak.Client(os.environ["NOBREAK_API_URL"], os.environ["NOBREAK_MODE"])
tester = nobreak.Tester(client)

for ob in bpy.data.objects:
    tester.test(ob.name, ob.location.x)
