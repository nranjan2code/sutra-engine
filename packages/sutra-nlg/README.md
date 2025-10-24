# Sutra NLG

**Grounded natural language generation without LLMs.**

Version: 1.0.0 | Language: Python | License: MIT

---

## Overview

Template-driven NLG that generates human-like responses from graph reasoning results without requiring LLMs.

### Key Features

- **📝 Template-Based**: No LLM required
- **🎭 Multiple Tones**: Friendly, formal, concise, regulatory
- **🔧 Grounded**: Only uses retrieved facts
- **⚡ Fast**: <10ms generation time

---

## Quick Start

```python
from sutra_nlg import NLGGenerator

nlg = NLGGenerator(tone="friendly")

# Generate from facts
facts = {"location": "Paris", "landmark": "Eiffel Tower"}
response = nlg.generate("location_query", facts)
print(response)
# "The Eiffel Tower is in Paris! 🗼"
```

---

## API Reference

```python
from sutra_nlg import NLGGenerator

generator = NLGGenerator(
    tone="friendly",  # friendly, formal, concise, regulatory
    include_emoji=True,
)

# Generate response
response = generator.generate(
    template_name="explain",
    facts={"concept": "AI", "definition": "..."},
)
```

---

## Templates

- `location_query`: Where is X?
- `definition`: What is X?
- `comparison`: X vs Y
- `causal`: Why does X happen?
- `temporal`: When did X occur?

---

## License

MIT License

**Built with ❤️ by the Sutra AI Team**
