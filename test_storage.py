
import socket
import struct
import json
import time

HOST = '127.0.0.1'
PORT = 9000

def send_nl_command(command):
    """Send a natural language command (text mode)"""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.connect((HOST, PORT))
        s.sendall(f"{command}\n".encode('utf-8'))
        response = s.recv(4096).decode('utf-8')
        return response

def test_nl_interface():
    print("\n--- Testing NL Interface ---")
    
    # Test 1: Learn
    print("Test 1: Remember 'Sutra is fast'")
    resp = send_nl_command("Remember that Sutra is fast")
    print(f"Response: {resp.strip()}")
    assert "LearnConceptV2Ok" in resp, "Failed to learn"

    # Test 2: Query
    print("Test 2: Search for 'Sutra'")
    time.sleep(1) # wait for indexing (though it's fast)
    resp = send_nl_command("Search for Sutra")
    print(f"Response: {resp.strip()}")
    # Note: might fail if embedding dim mismatch, but checking response format
    assert "QueryConceptOk" in resp, "Failed to query"

    # Test 3: List
    print("Test 3: List memory")
    resp = send_nl_command("ls")
    print(f"Response: {resp.strip()}")
    assert "ListRecentOk" in resp, "Failed to list"

    # Test 4: Garbage input
    print("Test 4: Garbage Input")
    resp = send_nl_command("blah blah blah")
    print(f"Response: {resp.strip()}")
    assert "Error" in resp, "Garbage input should return Error"

def run_tests():
    try:
        test_nl_interface()
        print("\n✅ All Tests Passed!")
    except Exception as e:
        print(f"\n❌ Test Failed: {e}")
        exit(1)

if __name__ == "__main__":
    run_tests()
