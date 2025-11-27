import re

def fix_environment_scope(content):
    """Fix Environment scope issues for ort 1.16"""
    
    # Add Environment to the Embedder struct
    # Find the struct definition
    struct_pattern = r'(pub struct Embedder \{[^}]+)'
    
    def add_env_field(match):
        struct_content = match.group(1)
        if 'environment:' not in struct_content:
            # Add environment field to the struct
            struct_content = struct_content.replace(
                'config: EmbedderConfig,',
                'config: EmbedderConfig,\n    environment: Arc<Environment>,'
            )
        return struct_content
    
    content = re.sub(struct_pattern, add_env_field, content, flags=re.MULTILINE | re.DOTALL)
    
    # Fix the constructor to store the environment
    content = re.sub(
        r'let _env = Environment::builder\(\)\.with_name\("sutra-embedder"\)\.build\(\)\?;',
        'let environment = Arc::new(Environment::builder().with_name("sutra-embedder").build()?);',
        content
    )
    
    # Add environment to struct initialization
    content = re.sub(
        r'let mut embedder = Self \{\s*config,',
        'let mut embedder = Self {\n            config,\n            environment: environment.clone(),',
        content
    )
    
    # Fix session builder to use self.environment
    content = re.sub(
        r'let session_builder = SessionBuilder::new\(&_env\)\?',
        'let session_builder = SessionBuilder::new(&self.environment)?',
        content
    )
    
    return content

# Read and fix the file
with open('src/embedder.rs', 'r') as f:
    content = f.read()

fixed_content = fix_environment_scope(content)

with open('src/embedder.rs', 'w') as f:
    f.write(fixed_content)

print("Fixed Environment scope for ort 1.16")
