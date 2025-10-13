#!/usr/bin/env python3
"""Quick import test for biological intelligence components."""

import sys
print('Python version:', sys.version)

try:
    from src.biological_trainer import BiologicalTrainer
    print('✅ BiologicalTrainer imported successfully')
except Exception as e:
    print('❌ BiologicalTrainer import failed:', e)

try:
    import biological_service
    print('✅ biological_service imported successfully')
except Exception as e:
    print('❌ biological_service import failed:', e)

try:
    from biological_service import create_api_server, BiologicalIntelligenceService
    print('✅ API components imported successfully')
except Exception as e:
    print('❌ API components import failed:', e)

print('\n🧠 Testing service initialization...')
try:
    service = biological_service.BiologicalIntelligenceService(workspace_path="./test_workspace")
    print('✅ Service initialization successful')
except Exception as e:
    print('❌ Service initialization failed:', e)