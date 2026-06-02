#!/usr/bin/env python3
print("Status: 200 OK")
print("Content-Type: text/html")
print("Set-Cookie: admin_auth=1; Path=/; HttpOnly")
print("")
# add href to /admin in the html response
print("<html><body><h1>Cookie was set.</h1><p>Go back to <a href='/admin'>/admin</a>.</p></body></html>")
