import re

def fix_inputs_macro_carefully(filepath):
    """Fix ort::inputs! macro usage for ort 1.16"""
    
    with open(filepath, 'r') as f:
        content = f.read()
    
    # Replace ort::inputs![ ... ] with vec![ ... ]
    # The inputs format changed in ort 1.16
    content = re.sub(
        r'ort::inputs!\[',
        'vec![',
        content
    )
    
    with open(filepath, 'w') as f:
        f.write(content)

fix_inputs_macro_carefully('src/embedder.rs')
print("Fixed ort::inputs! macro usage")
