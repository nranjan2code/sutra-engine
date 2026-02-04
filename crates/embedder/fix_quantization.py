import re

def fix_quantization_ort_16(content):
    """Fix quantization.rs for ort 1.16 API"""
    
    # Fix environment reference type
    content = re.sub(
        r'let _env = Environment::builder\(\)\.with_name\("sutra"\)\.build\(\)\?;',
        'let _env = Arc::new(Environment::builder().with_name("sutra").build()?);',
        content
    )
    
    # Fix thread count type
    content = re.sub(
        r'\.with_intra_threads\(rayon::current_num_threads\(\)\)',
        '.with_intra_threads(rayon::current_num_threads() as i16)',
        content
    )
    
    # Fix session builder API
    content = re.sub(
        r'\.commit_from_file\(model_path\)\?;',
        '.commit_from_file(model_path.as_ref())?;',
        content
    )
    
    return content

# Read and fix quantization.rs
with open('src/quantization.rs', 'r') as f:
    content = f.read()

fixed_content = fix_quantization_ort_16(content)

with open('src/quantization.rs', 'w') as f:
    f.write(fixed_content)

print("Fixed quantization.rs for ort 1.16 API")
