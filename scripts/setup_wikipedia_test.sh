#!/bin/bash
# Setup script for Wikipedia Performance Suite

echo "🌍 Setting up Wikipedia Performance Test Suite"
echo ""

# Check if venv is activated
if [ -z "$VIRTUAL_ENV" ]; then
    echo "⚠️  Virtual environment not activated!"
    echo "Please run: source venv/bin/activate"
    exit 1
fi

# Install datasets library
echo "📦 Installing Hugging Face datasets..."
pip install datasets

echo ""
echo "✅ Setup complete!"
echo ""
echo "📝 Next steps:"
echo ""
echo "1. (Optional) Set your Hugging Face token to avoid rate limits:"
echo "   export HF_TOKEN='your_token_here'"
echo ""
echo "2. Run the test:"
echo "   python scripts/wikipedia_performance_suite.py 100   # 100 articles"
echo "   python scripts/wikipedia_performance_suite.py 1000  # 1000 articles"
echo ""
echo "💡 The test will:"
echo "   - Download real Wikipedia articles"
echo "   - Learn from them (creating concepts & associations)"
echo "   - Test question answering"
echo "   - Save results to performance_results/"
echo ""
