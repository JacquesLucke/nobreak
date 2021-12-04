import sys
import os
import requests

server_url = os.environ["NOBREAK_SERVER_URL"]
res = requests.get(server_url).text
