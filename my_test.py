import sys
import os
import requests
from pprint import pprint
import urllib.parse

server_url = os.environ["NOBREAK_SERVER_URL"]
index_res = requests.get(server_url).json()
print(index_res)

log_url = urllib.parse.urljoin(server_url, index_res["log"])
get_url = urllib.parse.urljoin(server_url, index_res["get"])
print(log_url)

key = "My Key"
value = b"My Value"

requests.post(log_url + f"/{key}", data=value)

result = requests.get(get_url + f"/{key}")
print("The result:", result.content)
