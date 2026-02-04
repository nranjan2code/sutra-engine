import re

def fix_ort_16_complete_api(content):
    """Complete fix for ort 1.16 API compatibility"""
    
    # 1. Fix Value::from_array calls to include allocator
    # Pattern: Value::from_array(array) -> Value::from_array(allocator, array)
    content = re.sub(
        r'Value::from_array\(([^)]+)\)',
        r'Value::from_array(ort::AllocatorType::Arena, \1)',
        content
    )
    
    # 2. Fix duplicate environment field in struct
    # Remove duplicate environment lines
    lines = content.split('\n')
    new_lines = []
    seen_env_field = False
    
    for line in lines:
        if 'environment: environment.clone(),' in line:
            if not seen_env_field:
                new_lines.append(line)
                seen_env_field = True
            # Skip duplicate lines
        else:
            new_lines.append(line)
    
    content = '\n'.join(new_lines)
    
    # 3. Fix session output access (.1 field doesn't exist in 1.16)
    content = re.sub(
        r'outputs\.iter\(\)\.next\(\)\.ok_or_else\([^)]+\)\?\.1',
        'outputs.into_iter().next().ok_or_else(|| anyhow!("No output tensor"))?',
        content
    )
    
    content = re.sub(
        r'outputs\.iter\(\)\.next\(\)\.unwrap\(\)\.1',
        'outputs[0]',
        content
    )
    
    # 4. Fix thread count type conversion
    content = re.sub(
        r'\.with_intra_threads\(rayon::current_num_threads\(\)\)',
        '.with_intra_threads(rayon::current_num_threads() as i16)',
        content
    )
    
    # 5. Remove unsupported log level API
    content = re.sub(
        r'\.with_log_level\(LogLevel::Fatal\)\?',
        '',
        content
    )
    
    # 6. Fix try_extract_tensor method name
    content = re.sub(
        r'\.try_extract_tensor::<f32>\(\)',
        '.try_extract::<f32>()',
        content
    )
    
    return content

# Read and fix embedder.rs
with open('src/embedder.rs', 'r') as f:
    content = f.read()

fixed_content = fix_ort_16_complete_api(content)

with open('src/embedder.rs', 'w') as f:
    f.write(fixed_content)

print("Applied complete ort 1.16 API fixes to embedder.rs")
