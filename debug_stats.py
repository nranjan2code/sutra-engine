#!/usr/bin/env python3

import socket
import struct
import msgpack

# Connect to storage server directly
sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
sock.connect(('localhost', 50051))

# Send GetStats request
request = 'GetStats'
packed = msgpack.packb(request)
sock.sendall(struct.pack('>I', len(packed)))
sock.sendall(packed)

# Receive response
length_bytes = sock.recv(4)
length = struct.unpack('>I', length_bytes)[0]
response_bytes = sock.recv(length)
response = msgpack.unpackb(response_bytes, raw=False)

print('Raw storage server stats response:')
print(response)
sock.close()