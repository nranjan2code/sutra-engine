import re

def fix_ort_inputs_macro(content):
    """Replace ort::inputs! macro with proper input creation for ort 1.16"""
    
    # Pattern to match ort::inputs![ ... ] with multiline content
    pattern = r'ort::inputs!\[\s*([^]]+)\s*\]'
    
    def replace_inputs(match):
        inputs_content = match.group(1).strip()
        # For ort 1.16, we need to create inputs differently
        # Convert "name" => value to proper input format
        lines = [line.strip() for line in inputs_content.split(',') if line.strip()]
        
        # Create vec of input values 
        input_pairs = []
        for line in lines:
            if '=>' in line:
                parts = line.split('=>')
                if len(parts) == 2:
                    name = parts[0].strip().strip('"')
                    value = parts[1].strip().rstrip('?')
                    input_pairs.append(value)
        
        if input_pairs:
            return f"vec![{', '.join(input_pairs)}]"
        else:
            return "vec![]"
    
    # Apply the replacement
    content = re.sub(pattern, replace_inputs, content, flags=re.MULTILINE | re.DOTALL)
    
    return content

# Read and fix the file
with open('src/embedder.rs', 'r') as f:
    content = f.read()

fixed_content = fix_ort_inputs_macro(content)

with open('src/embedder.rs', 'w') as f:
    f.write(fixed_content)

print("Fixed ort::inputs! macro usage for ort 1.16")
