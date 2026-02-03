import time
from sutra_engine_client import SutraClient

def main():
    # 1. Initialize Client
    print("🔌 Connecting to Sutra Engine...")
    client = SutraClient(host="localhost", port=50051)
    
    try:
        # 2. Ingest Knowledge
        print("🧠 Ingesting knowledge...")
        concept_id = client.learn(
            "The Sutra Engine is a high-performance reasoning core.",
            generate_embedding=False # Disabled because we might not have embedding service running
        )
        print(f"✅ Ingested concept ID: {concept_id}")
        
        # 3. Search
        print("🔍 Searching for 'performance'...")
        results = client.search("performance", limit=3)
        print(f"Found {len(results)} results:")
        for res in results:
            print(f" - ID: {res['id']} (Score: {res['score']:.4f})")
            
        # 4. Get Stats
        stats = client.get_stats()
        print(f"📊 Engine Stats: {stats}")
        
    except Exception as e:
        print(f"❌ Error: {e}")
    finally:
        client.close()

if __name__ == "__main__":
    main()
