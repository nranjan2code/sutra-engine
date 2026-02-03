from sutra_engine_client import SutraClient
import time
import os

def main():
    # Use default port 50051 or from env
    port = int(os.environ.get("STORAGE_PORT", 50051))
    
    print(f"Connecting to Sutra Engine on port {port}...")
    
    # 1. Initialize Client
    try:
        client = SutraClient(host="localhost", port=port)
    except Exception as e:
        print(f"Failed to connect: {e}")
        return

    # 2. Learn a Concept
    text = "Sutra Engine is a lightweight, embedded AI memory system."
    print(f"\n🧠 Learning: '{text}'")
    
    concept_id = client.learn(
        text,
        generate_embedding=False # Set True if embedding model is running
    )
    print(f"✅ Stored with ID: {concept_id}")

    # 3. Retrieve it back
    print(f"\n🔍 Retrieving {concept_id}...")
    concept = client.get(concept_id)
    
    if concept:
        print(f"   Found: {concept.get('content')}")
        print(f"   Confidence: {concept.get('confidence')}")
    else:
        print("❌ Concept not found")

    # 4. Search (Text match since embedding is off in this example)
    print("\n🔎 Searching for 'Sutra'...")
    results = client.search("Sutra", limit=5)
    
    for res in results:
        print(f"   Match: {res}")

if __name__ == "__main__":
    main()
