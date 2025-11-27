#!/usr/bin/env python3
"""Fix ndarray to OwnedTensorArrayData conversion for ort 2.0"""

import re

def fix_embedder_file():
    # Read the file
    with open('src/embedder.rs', 'r') as f:
        content = f.read()
    
    # Pattern to match Value::from_array calls with ArrayBase types
    # Need to convert: .to_owned() to proper format for ort 2.0
    
    # Replace all instances of Value::from_array(variable.to_owned()) with proper conversion
    patterns = [
        # Pattern 1: Value::from_array(variable.to_owned())
        (r'Value::from_array\(([^.]+)\.to_owned\(\)\)', 
         r'Value::from_array((\1.shape().to_vec(), \1.into_raw_vec().into_boxed_slice()))'),
        
        # Pattern 2: Value::from_array(variable.clone().to_owned())  
        (r'Value::from_array\(([^.]+)\.clone\(\)\.to_owned\(\)\)',
         r'Value::from_array((\1.shape().to_vec(), \1.clone().into_raw_vec().into_boxed_slice()))'),
    ]
    
    for pattern, replacement in patterns:
        content = re.sub(pattern, replacement, content)
    
    # Write the file back
    with open('src/embedder.rs', 'w') as f:
        f.write(content)
    
    print("Fixed Value::from_array conversions in embedder.rs")

def fix_quantization_file():
    # Read the file
    with open('src/quantization.rs', 'r') as f:
        content = f.read()
    
    # Fix quantization.rs issues for ort 2.0
    fixes = [
        # Environment creation
        (r'Environment::builder\(\)\.with_name\("sutra"\)\.build\(\)\?',
         'Environment::new().unwrap()'),
        
        # SessionBuilder::new doesn't take environment in ort 2.0
        (r'SessionBuilder::new\(&_env\)\?',
         'SessionBuilder::new()?'),
        
        # Fix thread count type (i16 -> usize)
        (r'\(rayon::current_num_threads\(\) as i16\)',
         'rayon::current_num_threads()'),
        
        # Replace with_model_from_file with commit_from_file
        (r'\.with_model_from_file\(model_path\)\?\.commit\(\)\?',
         '.commit_from_file(model_path)?'),
    ]
    
    for pattern, replacement in fixes:
        content = re.sub(pattern, replacement, content)
    
    # Write the file back
    with open('src/quantization.rs', 'w') as f:
        f.write(content)
    
    print("Fixed quantization.rs for ort 2.0 API")

if __name__ == "__main__":
    fix_embedder_file()
    fix_quantization_file()
    print("Applied all ort 2.0 compatibility fixes")
