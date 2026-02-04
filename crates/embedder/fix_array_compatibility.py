import re

def fix_ort_array_compatibility(content):
    """Fix ndarray to ort Value::from_array compatibility for ort 2.0"""
    
    # Convert ArrayBase to compatible format by converting to owned data
    # The ort 2.0 API expects data that implements OwnedTensorArrayData
    # We need to convert: array -> array.to_owned() or similar
    
    # Pattern: Value::from_array(array_name) -> Value::from_array(array_name.to_owned())
    content = re.sub(
        r'Value::from_array\(([a-zA-Z_][a-zA-Z0-9_]*(?:\.clone\(\))?)\)',
        r'Value::from_array(\1.to_owned())',
        content
    )
    
    # For cases where we have method calls on arrays like .clone()
    # We need to be more careful with the conversion
    
    return content

def fix_imports(content):
    """Fix import issues in other files"""
    
    # Fix flash_attention.rs imports
    content = re.sub(
        r'use ort::\{Environment, SessionBuilder\};',
        'use ort::session::builder::SessionBuilder;',
        content
    )
    
    # Fix quantization.rs imports  
    content = re.sub(
        r'use ort::\{Environment, GraphOptimizationLevel, Session, SessionBuilder\};',
        'use ort::session::{builder::GraphOptimizationLevel, builder::SessionBuilder, Session};',
        content
    )
    
    # Fix quantization.rs syntax error
    content = re.sub(
        r'\.with_model_from_file\(model_path\.as_ref\(\)\?\.commit\(\)\)\?;',
        '.with_model_from_file(model_path)?.commit()?;',
        content
    )
    
    return content

# Fix embedder.rs
with open('src/embedder.rs', 'r') as f:
    content = f.read()

fixed_content = fix_ort_array_compatibility(content)
with open('src/embedder.rs', 'w') as f:
    f.write(fixed_content)

# Fix other files
for filename in ['src/flash_attention.rs', 'src/quantization.rs']:
    with open(filename, 'r') as f:
        content = f.read()
    
    fixed_content = fix_imports(content)
    with open(filename, 'w') as f:
        f.write(fixed_content)

print("Fixed array compatibility and imports for ort 2.0")
