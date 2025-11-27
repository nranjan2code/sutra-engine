#!/bin/bash

# Enhanced Model Download Helper Script
# Downloads latest AI models including DeepSeek and Llama for comprehensive testing

set -e

echo "╔══════════════════════════════════════════════════════╗"
echo "║        🌐 SutraWorks Enhanced Model Downloader 🌐   ║"
echo "║     Download latest models for comprehensive testing  ║"
echo "╚══════════════════════════════════════════════════════╝"
echo

# Load environment variables securely
if [ -f ".env" ]; then
    echo "🔐 Loading secure configuration..."
    export $(grep -v '^#' .env | xargs)
else
    echo "⚠️  No .env file found - some downloads may require authentication"
fi

# Function to download with HuggingFace authentication
download_model_with_auth() {
    local repo=$1
    local model_name=$2
    local requires_auth=$3
    local size_estimate=$4
    local cache_dir="$HOME/.cache/sutraworks/models/$repo"
    
    echo "📥 Downloading $model_name ($size_estimate)..."
    echo "Repository: $repo"
    echo "Cache directory: $cache_dir"
    
    if [ "$requires_auth" = "true" ] && [ -z "$HUGGINGFACE_TOKEN" ]; then
        echo "🔒 This model requires authentication. Please ensure your HF token is set in .env file."
        echo "⏭️  Skipping $model_name"
        echo
        return
    fi
    
    mkdir -p "$cache_dir"
    
    if command -v git-lfs >/dev/null 2>&1; then
        echo "✅ Git LFS detected - using for efficient download"
        
        # Set up git credentials for private repos
        if [ "$requires_auth" = "true" ] && [ -n "$HUGGINGFACE_TOKEN" ]; then
            git config --global credential."https://huggingface.co".helper store
            echo "https://oauth:$HUGGINGFACE_TOKEN@huggingface.co" >> ~/.git-credentials
        fi
        
        if [ ! -d "$cache_dir/.git" ]; then
            git clone https://huggingface.co/$repo "$cache_dir"
        else
            echo "📁 Repository already exists, updating..."
            cd "$cache_dir"
            git pull
            cd - >/dev/null
        fi
        
        echo "✅ Downloaded $model_name to $cache_dir"
    else
        echo "⚠️  Git LFS not found - install for better performance"
        echo "Install: brew install git-lfs (macOS) or apt-get install git-lfs (Ubuntu)"
        echo "Alternative: Using huggingface-cli (requires: pip install huggingface_hub)"
        
        if command -v huggingface-cli >/dev/null 2>&1; then
            if [ "$requires_auth" = "true" ] && [ -n "$HUGGINGFACE_TOKEN" ]; then
                huggingface-cli login --token "$HUGGINGFACE_TOKEN"
            fi
            huggingface-cli download "$repo" --local-dir "$cache_dir"
        else
            echo "⚠️  Consider installing huggingface-cli: pip install huggingface_hub"
        fi
    fi
    echo
}

# Check available disk space
available_space=$(df -h . | tail -1 | awk '{print $4}')
echo "💾 Available disk space: $available_space"
echo "⚠️  Note: Some large models (7B+) require significant space (10GB+)"
echo

# Enhanced model catalog with latest models
echo "📋 Available models for download:"
echo
echo "🔥 NEWEST MODELS (2024-2025):"
echo "1.  DeepSeek-Coder-V2 1.3B (~2.6GB) - Latest coding model"
echo "2.  DeepSeek-Coder-V2 6.7B (~13GB) - Advanced coding model"
echo "3.  Llama 3.2 1B (~2GB) - Latest Meta small model"
echo "4.  Llama 3.2 3B (~6GB) - Latest Meta medium model"
echo "5.  Llama 3.1 8B (~16GB) - Latest Meta large model ⚠️ Large"
echo
echo "📚 ESTABLISHED MODELS:"
echo "6.  RWKV-4 169M (~700MB) - Lightweight RNN language model"
echo "7.  RWKV-4 430M (~1.7GB) - Medium RNN language model"  
echo "8.  Mamba 130M (~500MB) - State-space model"
echo "9.  Mamba 370M (~1.5GB) - Larger state-space model"
echo
echo "🎁 BUNDLE OPTIONS:"
echo "10. Latest Small Bundle (DeepSeek 1.3B + Llama 3.2 1B = ~5GB)"
echo "11. Comprehensive Bundle (All newest models ~37GB) ⚠️ Very Large"
echo "12. Original Testing Bundle (RWKV 169M + Mamba 130M = ~1.2GB)"
echo "13. All established models (~4GB)"
echo
echo "0.  Skip download (test with synthetic data only)"
echo

read -p "Select models to download (0-13): " choice

case $choice in
    1)
        download_model_with_auth "deepseek-ai/deepseek-coder-1.3b-instruct" "DeepSeek-Coder-V2 1.3B" "false" "2.6GB"
        ;;
    2)
        download_model_with_auth "deepseek-ai/deepseek-coder-6.7b-instruct" "DeepSeek-Coder-V2 6.7B" "false" "13GB"
        ;;
    3)
        download_model_with_auth "meta-llama/Llama-3.2-1B-Instruct" "Llama 3.2 1B" "true" "2GB"
        ;;
    4)
        download_model_with_auth "meta-llama/Llama-3.2-3B-Instruct" "Llama 3.2 3B" "true" "6GB"
        ;;
    5)
        download_model_with_auth "meta-llama/Llama-3.1-8B-Instruct" "Llama 3.1 8B" "true" "16GB"
        ;;
    6)
        download_model_with_auth "BlinkDL/rwkv-4-pile-169m" "RWKV-4 169M" "false" "700MB"
        ;;
    7)
        download_model_with_auth "BlinkDL/rwkv-4-pile-430m" "RWKV-4 430M" "false" "1.7GB"
        ;;
    8)
        download_model_with_auth "state-spaces/mamba-130m" "Mamba 130M" "false" "500MB"
        ;;
    9)
        download_model_with_auth "state-spaces/mamba-370m" "Mamba 370M" "false" "1.5GB"
        ;;
    10)
        echo "🎁 Downloading Latest Small Bundle..."
        download_model_with_auth "deepseek-ai/deepseek-coder-1.3b-instruct" "DeepSeek-Coder-V2 1.3B" "false" "2.6GB"
        download_model_with_auth "meta-llama/Llama-3.2-1B-Instruct" "Llama 3.2 1B" "true" "2GB"
        ;;
    11)
        echo "🎁 Downloading Comprehensive Bundle (this will take a while)..."
        download_model_with_auth "deepseek-ai/deepseek-coder-1.3b-instruct" "DeepSeek-Coder-V2 1.3B" "false" "2.6GB"
        download_model_with_auth "deepseek-ai/deepseek-coder-6.7b-instruct" "DeepSeek-Coder-V2 6.7B" "false" "13GB"
        download_model_with_auth "meta-llama/Llama-3.2-1B-Instruct" "Llama 3.2 1B" "true" "2GB"
        download_model_with_auth "meta-llama/Llama-3.2-3B-Instruct" "Llama 3.2 3B" "true" "6GB"
        download_model_with_auth "meta-llama/Llama-3.1-8B-Instruct" "Llama 3.1 8B" "true" "16GB"
        ;;
    12)
        echo "🎁 Downloading Original Testing Bundle..."
        download_model_with_auth "BlinkDL/rwkv-4-pile-169m" "RWKV-4 169M" "false" "700MB"
        download_model_with_auth "state-spaces/mamba-130m" "Mamba 130M" "false" "500MB"
        ;;
    13)
        echo "🎁 Downloading All Established Models..."
        download_model_with_auth "BlinkDL/rwkv-4-pile-169m" "RWKV-4 169M" "false" "700MB"
        download_model_with_auth "BlinkDL/rwkv-4-pile-430m" "RWKV-4 430M" "false" "1.7GB"
        download_model_with_auth "state-spaces/mamba-130m" "Mamba 130M" "false" "500MB"
        download_model_with_auth "state-spaces/mamba-370m" "Mamba 370M" "false" "1.5GB"
        ;;
    0)
        echo "⏭️  Skipping downloads - will use synthetic data for testing"
        echo
        ;;
    *)
        echo "❌ Invalid selection"
        exit 1
        ;;
esac

echo "🎯 Next steps:"
echo
echo "1. Quick validation (no downloads needed):"
echo "   cargo run --example manual_test --release"
echo
echo "2. Test with downloaded models:"
echo "   cargo run --example comprehensive_validation --release"
echo
echo "3. Test new models specifically:"
echo "   cargo run --example deepseek_validation --release"
echo "   cargo run --example llama_validation --release"
echo
echo "4. Run full end-to-end pipeline:"
echo "   cargo run --example end_to_end --release"
echo

# Enhanced model detection and summary
cache_dir="$HOME/.cache/sutraworks/models"
if [ -d "$cache_dir" ]; then
    echo "📁 Downloaded models summary:"
    total_size=0
    find "$cache_dir" -name "*.safetensors" -o -name "*.bin" | while read -r file; do
        size=$(ls -lh "$file" | awk '{print $5}')
        model_name=$(basename "$(dirname "$file")")
        echo "  • $model_name: $size"
    done
    echo
    
    # Check for specific model types
    if find "$cache_dir" -path "*deepseek*" -name "*.safetensors" | grep -q .; then
        echo "🔥 DeepSeek models ready for testing!"
    fi
    
    if find "$cache_dir" -path "*llama*" -name "*.safetensors" | grep -q .; then
        echo "🦙 Llama models ready for testing!"
    fi
    
    if find "$cache_dir" -path "*rwkv*" -name "*.safetensors" | grep -q .; then
        echo "🐦 RWKV models ready for testing!"
    fi
    
    if find "$cache_dir" -path "*mamba*" -name "*.safetensors" | grep -q .; then
        echo "🐍 Mamba models ready for testing!"
    fi
    echo
fi

echo "✅ Enhanced setup complete! Ready for cutting-edge AI model testing."
echo "🔒 Your HuggingFace token is securely stored in .env (excluded from git)"
echo
echo "💡 Pro tip: Start with option 10 (Latest Small Bundle) for best balance of"
echo "   performance and download time, then explore larger models as needed."