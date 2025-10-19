#!/usr/bin/env python3

import sys
sys.path.append('packages/sutra-storage-client-tcp')

from sutra_storage_client import StorageClient

# Get storage client and check what stats() returns
client = None
try:
    client = StorageClient('localhost:50051')
    raw_stats = client.stats()
    print('Raw client stats:', raw_stats)
    print('Type:', type(raw_stats))
    print('Keys:', raw_stats.keys() if isinstance(raw_stats, dict) else 'Not a dict')
    
    # Check specific keys the API expects
    concepts = raw_stats.get("concepts", "NOT_FOUND")
    edges = raw_stats.get("edges", "NOT_FOUND") 
    print(f'concepts: {concepts}')
    print(f'edges: {edges}')
    
except Exception as e:
    print('Error:', e)
    import traceback
    traceback.print_exc()
finally:
    if client:
        client.close()
