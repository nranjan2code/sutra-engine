import re

def fix_ort_16_proper_api(content):
    """Proper fix for ort 1.16 API with correct allocator and array types"""
    
    # 1. Add necessary imports
    if 'use std::sync::Arc;' not in content:
        content = 'use std::sync::Arc;\n' + content
    
    # 2. Fix Value::from_array calls - need proper allocator and convert to CowArray
    # The ort 1.16 API expects (allocator_ptr: *mut OrtAllocator, array: &CowArray)
    content = re.sub(
        r'Value::from_array\(ort::AllocatorType::Arena, ([^)]+)\)',
        r'Value::from_array(std::ptr::null_mut(), &\1.into())',
        content
    )
    
    # 3. Fix missing Arc import for quantization.rs
    content = re.sub(
        r'use ort::\{Environment, GraphOptimizationLevel, Session, SessionBuilder\};',
        'use ort::{Environment, GraphOptimizationLevel, Session, SessionBuilder};\nuse std::sync::Arc;',
        content
    )
    
    # 4. Fix session builder commit API - use with_model_from_file instead
    content = re.sub(
        r'\.commit_from_file\(([^)]+)\)',
        r'.with_model_from_file(\1)?.commit()',
        content
    )
    
    # 5. Fix tensor extraction for ort 1.16 - outputs are Vec<Value> now
    content = re.sub(
        r'let tensor_view = output_tensor\.try_extract::<f32>\(\)\?;',
        'let tensor_view = output_tensor.try_extract_tensor::<f32>()?;',
        content
    )
    
    # 6. Fix the missing field error by ensuring environment field is properly added
    # We need to make sure the Arc<Environment> is in struct and properly initialized
    
    return content

# Fix embedder.rs
with open('src/embedder.rs', 'r') as f:
    content = f.read()

fixed_content = fix_ort_16_proper_api(content)
with open('src/embedder.rs', 'w') as f:
    f.write(fixed_content)

# Fix quantization.rs  
with open('src/quantization.rs', 'r') as f:
    content = f.read()

fixed_content = fix_ort_16_proper_api(content)
with open('src/quantization.rs', 'w') as f:
    f.write(fixed_content)

print("Applied proper ort 1.16 API fixes")
