#!/usr/bin/env python3
print("Status: 200 OK")
print("Content-Type: text/html")
print("Set-Cookie: admin_auth=1; Path=/; HttpOnly")
print("")
print("<html><body><h1>Cookie was set.</h1><p>Go back to /admin.</p></body></html>")
