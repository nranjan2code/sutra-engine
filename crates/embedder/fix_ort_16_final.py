import re

def apply_ort_16_fixes_carefully(filepath):
    """Apply ort 1.16 fixes more carefully to avoid syntax errors"""
    
    with open(filepath, 'r') as f:
        content = f.read()
    
    # 1. Fix imports first
    content = re.sub(
        r'use ort::logging::LogLevel;',
        '// use ort::logging::LogLevel; // Not available in 1.16',
        content
    )
    
    content = re.sub(
        r'use ort::session::\{builder::GraphOptimizationLevel, builder::SessionBuilder, Session\};',
        'use ort::{Environment, GraphOptimizationLevel, Session, SessionBuilder};',
        content
    )
    
    content = re.sub(
        r'use ort::value::Value;',
        'use ort::Value;',
        content
    )
    
    # Add Arc import if not present
    if 'use std::sync::Arc;' not in content:
        lines = content.split('\n')
        import_idx = 0
        for i, line in enumerate(lines):
            if line.startswith('use '):
                import_idx = i + 1
        lines.insert(import_idx, 'use std::sync::Arc;')
        content = '\n'.join(lines)
    
    # 2. Fix environment initialization and struct
    content = re.sub(
        r'ort::init\(\)\.with_name\("sutra-embedder"\)\.commit\(\)\?;',
        'let environment = Arc::new(Environment::builder().with_name("sutra-embedder").build()?);',
        content
    )
    
    # 3. Fix session builder
    content = re.sub(
        r'let mut session_builder = Session::builder\(\)\?',
        'let session_builder = SessionBuilder::new(&environment)?',
        content
    )
    
    # 4. Remove unsupported methods
    content = re.sub(
        r'\.with_log_level\(LogLevel::Fatal\)\?',
        '',
        content
    )
    
    # 5. Fix thread count
    content = re.sub(
        r'\.with_intra_threads\(rayon::current_num_threads\(\)\)',
        '.with_intra_threads(rayon::current_num_threads() as i16)',
        content
    )
    
    # 6. Fix session builder commit
    content = re.sub(
        r'\.commit_from_file\(',
        '.with_model_from_file(',
        content
    )
    
    content = re.sub(
        r'\.with_model_from_file\(([^)]+)\)\?;',
        r'.with_model_from_file(\1)?.commit()?;',
        content
    )
    
    # 7. Add environment to struct initialization where missing
    # Find the struct init that's missing environment field
    content = re.sub(
        r'(let mut embedder = Self \{\s+config,\s+)(session: None,)',
        r'\1environment: environment.clone(),\n            \2',
        content
    )
    
    # 8. Fix Value::from_array calls - do this very carefully
    # First, let's convert arrays to the right type and use null allocator
    content = re.sub(
        r'Value::from_array\(([^,)]+)\)',
        r'Value::from_array(std::ptr::null_mut(), &\1.view().into())',
        content
    )
    
    # 9. Fix try_extract_tensor method
    content = re.sub(
        r'\.try_extract_tensor::<f32>\(\)',
        '.try_extract::<f32>()',
        content
    )
    
    # 10. Fix output access pattern
    content = re.sub(
        r'outputs\.iter\(\)\.next\(\)\.unwrap\(\)\.1',
        'outputs[0]',
        content
    )
    
    content = re.sub(
        r'outputs\.iter\(\)\.next\(\)\.ok_or_else\([^)]+\)\?\.1',
        'outputs.into_iter().next().ok_or_else(|| anyhow!("No output tensor"))?',
        content
    )
    
    with open(filepath, 'w') as f:
        f.write(content)

# Apply fixes
apply_ort_16_fixes_carefully('src/embedder.rs')
print("Applied careful ort 1.16 fixes to embedder.rs")
