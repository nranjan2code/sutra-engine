//! Adapter interface for pluggable data sources
//! 
//! Supports: Files, Databases, Kafka, APIs, and any custom source

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
// use futures::Stream;  // Not needed for this implementation

/// Data item extracted from any source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataItem {
    /// Main content text
    pub content: String,
    
    /// Optional metadata
    pub metadata: HashMap<String, JsonValue>,
    
    /// Optional pre-computed embedding
    pub embedding: Option<Vec<f32>>,
    
    /// Source identifier
    pub source_id: String,
    
    /// Item type (article, record, message, etc.)
    pub item_type: String,
}

impl DataItem {
    pub fn size_bytes(&self) -> u64 {
        let metadata_size = serde_json::to_string(&self.metadata)
            .map(|s| s.len())
            .unwrap_or(0) as u64;
        
        self.content.len() as u64 + 
        metadata_size +
        self.embedding.as_ref().map(|e| e.len() * 4).unwrap_or(0) as u64
    }
}

/// Stream of data items from a source
#[async_trait]
pub trait DataStream: Send + Sync {
    /// Get next item from stream
    async fn next(&mut self) -> Option<Result<DataItem>>;
    
    /// Estimate total items (if known)
    async fn estimate_total(&self) -> Result<Option<u64>>;
    
    /// Get current position in stream
    fn position(&self) -> u64;
}

/// Main adapter interface - implement this for new data sources
#[async_trait]
pub trait IngestionAdapter: Send + Sync {
    /// Adapter name (e.g., "wikipedia", "postgres", "kafka")
    fn name(&self) -> &str;
    
    /// Supported source types (e.g., ["file"], ["database"], ["stream"])
    fn supported_types(&self) -> Vec<&str>;
    
    /// Validate configuration before creating stream
    async fn validate_config(&self, config: &JsonValue) -> Result<()>;
    
    /// Create data stream from configuration
    async fn create_stream(&self, config: &JsonValue) -> Result<Box<dyn DataStream>>;
    
    /// Get adapter metadata/info
    fn info(&self) -> AdapterInfo;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterInfo {
    pub name: String,
    pub description: String,
    pub version: String,
    pub supported_types: Vec<String>,
    pub config_schema: JsonValue,  // JSON Schema for validation
}

// Google Sheets adapter for financial data
pub mod google_sheets;

// Built-in Rust adapters for performance-critical sources
pub mod builtin {
    use super::*;
    use tokio::fs::File;
    use tokio::io::{AsyncBufReadExt, BufReader};
    use std::path::Path;
    use regex::Regex;
    
    /// High-performance file adapter
    pub struct FileAdapter {
        pub name: String,
    }
    
    impl FileAdapter {
        pub fn new() -> Self {
            Self {
                name: "file".to_string(),
            }
        }
    }
    
    #[async_trait]
    impl IngestionAdapter for FileAdapter {
        fn name(&self) -> &str {
            "file"
        }
        
        fn supported_types(&self) -> Vec<&str> {
            vec!["txt", "md", "json", "csv", "xml"]
        }
        
        async fn validate_config(&self, config: &JsonValue) -> Result<()> {
            let path = config.get("path")
                .and_then(|p| p.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'path' in config"))?;
                
            if !Path::new(path).exists() {
                return Err(anyhow::anyhow!("File does not exist: {}", path));
            }
            
            Ok(())
        }
        
        async fn create_stream(&self, config: &JsonValue) -> Result<Box<dyn DataStream>> {
            let path = config.get("path").unwrap().as_str().unwrap();
            let format = config.get("format").and_then(|f| f.as_str()).unwrap_or("auto");
            let separator = config.get("separator").and_then(|s| s.as_str());
            
            Ok(Box::new(FileStream::new(path, format, separator).await?))
        }
        
        fn info(&self) -> AdapterInfo {
            AdapterInfo {
                name: "file".to_string(),
                description: "High-performance file reader for txt, md, json, csv, xml".to_string(),
                version: "1.0.0".to_string(),
                supported_types: vec!["txt".to_string(), "md".to_string(), "json".to_string()],
                config_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "File path"},
                        "format": {"type": "string", "enum": ["auto", "wikipedia", "lines", "json"], "default": "auto"},
                        "separator": {"type": "string", "description": "Custom separator pattern"}
                    },
                    "required": ["path"]
                }),
            }
        }
    }
    
    /// File stream implementation
    pub struct FileStream {
        reader: BufReader<File>,
        #[allow(dead_code)]
        separator: Option<Regex>,
        format: String,
        position: u64,
        total_size: Option<u64>,
        buffer: String,
    }
    
    impl FileStream {
        async fn new(path: &str, format: &str, separator: Option<&str>) -> Result<Self> {
            let file = File::open(path).await?;
            let metadata = file.metadata().await?;
            let reader = BufReader::new(file);
            
            // Determine separator pattern based on format
            let separator_regex = match format {
                "wikipedia" => Some(Regex::new(r"\n\n\n+")?),
                "lines" => None, // Line by line
                _ => separator.map(|s| Regex::new(s)).transpose()?,
            };
            
            Ok(Self {
                reader,
                separator: separator_regex,
                format: format.to_string(),
                position: 0,
                total_size: Some(metadata.len()),
                buffer: String::new(),
            })
        }
    }
    
    #[async_trait]
    impl DataStream for FileStream {
        async fn next(&mut self) -> Option<Result<DataItem>> {
            match self.format.as_str() {
                "lines" => {
                    // Read line by line
                    self.buffer.clear();
                    match self.reader.read_line(&mut self.buffer).await {
                        Ok(0) => None, // EOF
                        Ok(_) => {
                            self.position += 1;
                            let content = self.buffer.trim().to_string();
                            if content.is_empty() {
                                return self.next().await; // Skip empty lines
                            }
                            
                            Some(Ok(DataItem {
                                content,
                                metadata: HashMap::new(),
                                embedding: None,
                                source_id: format!("line_{}", self.position),
                                item_type: "line".to_string(),
                            }))
                        }
                        Err(e) => Some(Err(anyhow::anyhow!("Read error: {}", e))),
                    }
                }
                "wikipedia" | _ => {
                    // Read with separator pattern (like Wikipedia articles)
                    // This would need more sophisticated implementation
                    // For now, just read lines
                    self.buffer.clear();
                    match self.reader.read_line(&mut self.buffer).await {
                        Ok(0) => None,
                        Ok(_) => {
                            self.position += 1;
                            Some(Ok(DataItem {
                                content: self.buffer.trim().to_string(),
                                metadata: HashMap::new(),
                                embedding: None,
                                source_id: format!("item_{}", self.position),
                                item_type: "article".to_string(),
                            }))
                        }
                        Err(e) => Some(Err(anyhow::anyhow!("Read error: {}", e))),
                    }
                }
            }
        }
        
        async fn estimate_total(&self) -> Result<Option<u64>> {
            // Rough estimate based on file size and average content size
            if let Some(size) = self.total_size {
                let avg_item_size = 2000; // Rough estimate
                Ok(Some(size / avg_item_size))
            } else {
                Ok(None)
            }
        }
        
        fn position(&self) -> u64 {
            self.position
        }
    }
}

// Python adapter bridge for maximum flexibility
#[cfg(feature = "python-plugins")]
pub mod python {
    use super::*;
    use pyo3::prelude::*;
    use pyo3_asyncio::tokio::future_into_py;
    
    /// Bridge to Python adapters
    pub struct PythonAdapter {
        py_object: PyObject,
        name: String,
    }
    
    impl PythonAdapter {
        pub fn new(py_object: PyObject, name: String) -> Self {
            Self { py_object, name }
        }
    }
    
    #[async_trait]
    impl IngestionAdapter for PythonAdapter {
        fn name(&self) -> &str {
            &self.name
        }
        
        fn supported_types(&self) -> Vec<&str> {
            // Query Python object for supported types
            Python::with_gil(|py| {
                self.py_object
                    .call_method0(py, "supported_types")
                    .and_then(|result| result.extract::<Vec<String>>(py))
                    .map(|types| types.iter().map(|s| s.as_str()).collect())
                    .unwrap_or_else(|_| vec!["unknown"])
            })
        }
        
        async fn validate_config(&self, config: &JsonValue) -> Result<()> {
            Python::with_gil(|py| {
                let config_str = serde_json::to_string(config)?;
                self.py_object
                    .call_method1(py, "validate_config", (config_str,))
                    .map_err(|e| anyhow::anyhow!("Python validation error: {}", e))?;
                Ok(())
            })
        }
        
        async fn create_stream(&self, config: &JsonValue) -> Result<Box<dyn DataStream>> {
            // This would create a Python stream wrapper
            // Implementation would be more complex in real code
            Err(anyhow::anyhow!("Python stream creation not implemented yet"))
        }
        
        fn info(&self) -> AdapterInfo {
            Python::with_gil(|py| {
                let info_dict = self.py_object
                    .call_method0(py, "info")
                    .and_then(|result| result.extract::<HashMap<String, PyObject>>(py))
                    .unwrap_or_default();
                
                AdapterInfo {
                    name: self.name.clone(),
                    description: "Python adapter".to_string(),
                    version: "1.0.0".to_string(),
                    supported_types: vec!["python".to_string()],
                    config_schema: serde_json::json!({}),
                }
            })
        }
    }
}