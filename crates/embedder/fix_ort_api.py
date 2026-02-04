import re

def fix_ort_16_api(content):
    """Fix ort API calls for version 1.16"""
    
    # Fix imports
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
    
    # Fix ort::init() calls
    content = re.sub(
        r'ort::init\(\)\.with_name\("[^"]+"\)\.commit\(\)\?;',
        'let _env = Environment::builder().with_name("sutra-embedder").build()?;',
        content
    )
    
    # Fix Session::builder() calls
    content = re.sub(
        r'let mut session_builder = Session::builder\(\)\?',
        'let session_builder = SessionBuilder::new(&_env)?',
        content
    )
    
    # Fix ort::inputs! macro calls - convert to array syntax
    content = re.sub(
        r'\.run\(ort::inputs\!\[([^\]]+)\]\?',
        r'.run(&[\1])?',
        content
    )
    
    content = re.sub(
        r'session\.run\(ort::inputs\!\[([^\]]+)\]\?',
        r'session.run(&[\1])?',
        content
    )
    
    # Fix tuple access for outputs (ort 1.16 returns Vec<Value>)
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
    
    return content

# Read and fix the file
with open('src/embedder.rs', 'r') as f:
    content = f.read()

fixed_content = fix_ort_16_api(content)

with open('src/embedder.rs', 'w') as f:
    f.write(fixed_content)

print("Fixed embedder.rs for ort 1.16 API")
