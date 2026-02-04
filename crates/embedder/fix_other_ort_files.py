import re

def fix_ort_16_api(content):
    """Fix ort API calls for version 1.16"""
    
    # Fix imports
    content = re.sub(
        r'use ort::session::\{builder::SessionBuilder\};',
        'use ort::{Environment, SessionBuilder};',
        content
    )
    
    content = re.sub(
        r'use ort::session::\{builder::GraphOptimizationLevel, Session\};',
        'use ort::{Environment, GraphOptimizationLevel, Session, SessionBuilder};',
        content
    )
    
    # Fix Session::builder() calls
    content = re.sub(
        r'let session = Session::builder\(\)\?',
        'let _env = Environment::builder().with_name("sutra").build()?;\n    let session = SessionBuilder::new(&_env)?',
        content
    )
    
    return content

# Fix flash_attention.rs
with open('src/flash_attention.rs', 'r') as f:
    content = f.read()
fixed_content = fix_ort_16_api(content)
with open('src/flash_attention.rs', 'w') as f:
    f.write(fixed_content)

# Fix quantization.rs  
with open('src/quantization.rs', 'r') as f:
    content = f.read()
fixed_content = fix_ort_16_api(content)
with open('src/quantization.rs', 'w') as f:
    f.write(fixed_content)

print("Fixed flash_attention.rs and quantization.rs for ort 1.16 API")
