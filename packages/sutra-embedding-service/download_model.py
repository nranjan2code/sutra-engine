#!/usr/bin/env python3
"""
Download nomic-embed-text-v1.5 model and save it locally
This eliminates the need to download during Docker container startup
"""

import os
from transformers import AutoTokenizer, AutoModel

def download_model():
    model_name = "nomic-ai/nomic-embed-text-v1.5"
    local_path = "./models/nomic-embed-text-v1.5"
    
    print(f"Downloading {model_name}...")
    print(f"Saving to: {local_path}")
    
    # Download and save tokenizer
    print("Downloading tokenizer...")
    tokenizer = AutoTokenizer.from_pretrained(model_name, trust_remote_code=True)
    tokenizer.save_pretrained(local_path)
    
    # Download and save model
    print("Downloading model...")
    model = AutoModel.from_pretrained(model_name, trust_remote_code=True)
    model.save_pretrained(local_path)
    
    print("✅ Model downloaded successfully!")
    print(f"Model saved to: {os.path.abspath(local_path)}")
    
    # Verify the download
    print("\nVerifying download...")
    files = os.listdir(local_path)
    print(f"Files: {files}")
    
    if "config.json" in files and "pytorch_model.bin" in files:
        print("✅ Download verification passed!")
    else:
        print("❌ Download verification failed!")

if __name__ == "__main__":
    download_model()