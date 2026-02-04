#!/usr/bin/env python3
"""Fix all ArrayBase to OwnedTensorArrayData compatibility issues"""

import re
import subprocess

def apply_fixes():
    # 1. Fix all Value::from_array calls to convert ndarray to compatible format
    sed_commands = [
        # Pattern: Variable.to_owned() -> Variable.into_raw_vec_and_offset().0.into_boxed_slice()
        r's/Value::from_array(\([^.]*\)\.to_owned())/Value::from_array((\1.shape().to_vec(), \1.into_raw_vec().into_boxed_slice()))/g',
        
        # Pattern: Variable.clone().to_owned() -> Variable.into_raw_vec_and_offset().0.into_boxed_slice()
        r's/Value::from_array(\([^.]*\)\.clone()\.to_owned())/Value::from_array((\1.shape().to_vec(), \1.clone().into_raw_vec().into_boxed_slice()))/g',
        
        # Alternative: Simple conversion via to_vec() for simple arrays
        r's/\.to_owned()\?\?/.to_vec()?/g',
    ]
    
    # Apply sed commands
    for cmd in sed_commands:
        result = subprocess.run(['sed', '-i', '', cmd, 'src/embedder.rs'], capture_output=True, text=True)
        if result.returncode != 0:
            print(f"Warning: sed command failed: {cmd}")
    
    # 2. Fix quantization.rs issues
    quantization_fixes = [
        # Environment builder pattern
        r's/Environment::builder().with_name("sutra").build()?;/Environment::new().unwrap();/g',
        
        # SessionBuilder::new no longer takes environment parameter
        r's/SessionBuilder::new(&_env)?/SessionBuilder::new()?/g',
        
        # Fix thread count type (i16 -> usize)
        r's/(rayon::current_num_threads() as i16)/(rayon::current_num_threads())/g',
        
        # Replace with_model_from_file with commit_from_file or create_from_file
        r's/\.with_model_from_file(model_path)?\.commit()?;/.commit_from_file(model_path)?;/g',
    ]
    
    for cmd in quantization_fixes:
        result = subprocess.run(['sed', '-i', '', cmd, 'src/quantization.rs'], capture_output=True, text=True)
        if result.returncode != 0:
            print(f"Warning: quantization sed command failed: {cmd}")
    
    print("Applied comprehensive ort 2.0 compatibility fixes")

if __name__ == "__main__":
    apply_fixes()
