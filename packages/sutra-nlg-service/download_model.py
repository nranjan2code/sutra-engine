#!/usr/bin/env python3
"""
Download and cache Gemma models locally
This eliminates the need to download during Docker container startup

Run this script once to cache models locally:
    python download_model.py --model google/gemma-3-270m-it
    python download_model.py --model google/gemma-2-2b-it

Requires HF_TOKEN environment variable for gated models
"""

import os
import argparse
from transformers import AutoTokenizer, AutoModelForCausalLM


def download_model(model_name: str, local_path: str = None):
    """
    Download and cache a Gemma model locally
    
    Args:
        model_name: HuggingFace model identifier (e.g., google/gemma-3-270m-it)
        local_path: Local directory to save the model (default: ./models/{model_name})
    """
    if local_path is None:
        # Extract model name without org prefix
        model_dir = model_name.split('/')[-1]
        local_path = f"./models/{model_dir}"
    
    # Create models directory if it doesn't exist
    os.makedirs(local_path, exist_ok=True)
    
    print(f"🔽 Downloading {model_name}...")
    print(f"📁 Saving to: {local_path}")
    
    # Check for HF token (required for gated models like Gemma)
    hf_token = os.getenv("HF_TOKEN")
    if not hf_token:
        print("⚠️  Warning: HF_TOKEN not found in environment")
        print("   Gemma models may require authentication")
        print("   Set HF_TOKEN in .env.local or environment")
    
    # Download and save tokenizer
    print("📦 Downloading tokenizer...")
    tokenizer = AutoTokenizer.from_pretrained(
        model_name,
        trust_remote_code=True,
        token=hf_token
    )
    tokenizer.save_pretrained(local_path)
    print("✅ Tokenizer saved")
    
    # Download and save model
    print("📦 Downloading model (this may take several minutes)...")
    model = AutoModelForCausalLM.from_pretrained(
        model_name,
        trust_remote_code=True,
        token=hf_token,
        torch_dtype="auto",  # Use appropriate dtype
    )
    model.save_pretrained(local_path)
    print("✅ Model saved")
    
    print(f"\n🎉 Model downloaded successfully!")
    print(f"📍 Model location: {os.path.abspath(local_path)}")
    
    # Verify the download
    print("\n🔍 Verifying download...")
    files = os.listdir(local_path)
    required_files = ["config.json", "tokenizer.json"]
    
    missing_files = [f for f in required_files if f not in files]
    if not missing_files:
        print("✅ Download verification passed!")
        print(f"📊 Total files: {len(files)}")
        
        # Calculate approximate size
        total_size = sum(
            os.path.getsize(os.path.join(local_path, f)) 
            for f in files 
            if os.path.isfile(os.path.join(local_path, f))
        )
        size_gb = total_size / (1024 ** 3)
        print(f"💾 Total size: {size_gb:.2f} GB")
    else:
        print(f"❌ Download verification failed! Missing: {missing_files}")
        return False
    
    return True


def main():
    parser = argparse.ArgumentParser(
        description="Download and cache Gemma models for Sutra NLG Service"
    )
    parser.add_argument(
        "--model",
        type=str,
        default="google/gemma-3-270m-it",
        help="HuggingFace model identifier (default: google/gemma-3-270m-it)"
    )
    parser.add_argument(
        "--output",
        type=str,
        default=None,
        help="Custom output directory (default: ./models/{model_name})"
    )
    parser.add_argument(
        "--all",
        action="store_true",
        help="Download all supported models (gemma-3-270m-it and gemma-2-2b-it)"
    )
    
    args = parser.parse_args()
    
    if args.all:
        # Download all supported models
        models = [
            "google/gemma-3-270m-it",
            "google/gemma-2-2b-it"
        ]
        print(f"📥 Downloading {len(models)} models...\n")
        
        for model in models:
            print(f"\n{'='*60}")
            success = download_model(model)
            if not success:
                print(f"❌ Failed to download {model}")
                return 1
        
        print(f"\n{'='*60}")
        print("🎉 All models downloaded successfully!")
        return 0
    else:
        # Download single model
        success = download_model(args.model, args.output)
        return 0 if success else 1


if __name__ == "__main__":
    exit(main())
