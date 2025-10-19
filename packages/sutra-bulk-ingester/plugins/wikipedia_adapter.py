#!/usr/bin/env python3
"""
Wikipedia Dataset Adapter

High-performance Wikipedia article extraction with intelligent chunking.
Supports both compressed and uncompressed Wikipedia dumps.
"""

import re
import json
import asyncio
from typing import Dict, List, Optional, AsyncIterator, Any
from pathlib import Path
import mmap
import gzip
import bz2


class WikipediaAdapter:
    """Optimized Wikipedia dataset adapter for bulk ingestion."""
    
    def __init__(self):
        self.name = "wikipedia"
        self.article_separator = re.compile(r'\n\n\n+')
        self.title_pattern = re.compile(r'^([^\n]+)$', re.MULTILINE)
        
    def supported_types(self) -> List[str]:
        """Return supported file types."""
        return ["txt", "gz", "bz2", "wikipedia"]
    
    def validate_config(self, config_str: str) -> None:
        """Validate adapter configuration."""
        config = json.loads(config_str)
        
        if "path" not in config:
            raise ValueError("Missing 'path' in configuration")
            
        path = Path(config["path"])
        if not path.exists():
            raise ValueError(f"File does not exist: {path}")
            
        if not path.is_file():
            raise ValueError(f"Path is not a file: {path}")
            
        # Check file size (warn if > 10GB)
        size_gb = path.stat().st_size / (1024**3)
        if size_gb > 10:
            print(f"Warning: Large file detected ({size_gb:.1f}GB)")
    
    def info(self) -> Dict[str, Any]:
        """Return adapter information."""
        return {
            "name": "wikipedia",
            "description": "High-performance Wikipedia dataset adapter",
            "version": "1.0.0",
            "supported_types": self.supported_types(),
            "config_schema": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string", 
                        "description": "Path to Wikipedia dataset file"
                    },
                    "min_article_length": {
                        "type": "integer", 
                        "default": 200,
                        "description": "Minimum article length in characters"
                    },
                    "max_article_length": {
                        "type": "integer", 
                        "default": 10000,
                        "description": "Maximum article length (longer articles are split)"
                    },
                    "compression": {
                        "type": "string", 
                        "enum": ["auto", "none", "gzip", "bz2"],
                        "default": "auto"
                    },
                    "encoding": {
                        "type": "string",
                        "default": "utf-8"
                    }
                },
                "required": ["path"]
            }
        }
    
    async def create_stream(self, config_str: str):
        """Create data stream from configuration."""
        config = json.loads(config_str)
        return WikipediaStream(config)


class WikipediaStream:
    """High-performance Wikipedia article stream."""
    
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.path = Path(config["path"])
        self.min_length = config.get("min_article_length", 200)
        self.max_length = config.get("max_article_length", 10000)
        self.encoding = config.get("encoding", "utf-8")
        
        # Auto-detect compression
        compression = config.get("compression", "auto")
        if compression == "auto":
            if self.path.suffix == ".gz":
                compression = "gzip"
            elif self.path.suffix == ".bz2":
                compression = "bz2"
            else:
                compression = "none"
        
        self.compression = compression
        self.position = 0
        self.total_size = self.path.stat().st_size
        self.file_handle = None
        self.buffer = ""
        self.article_count = 0
        
    async def __aenter__(self):
        """Async context manager entry."""
        await self.open()
        return self
        
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """Async context manager exit."""
        await self.close()
    
    async def open(self):
        """Open the file with appropriate decompression."""
        if self.compression == "gzip":
            self.file_handle = gzip.open(self.path, 'rt', encoding=self.encoding)
        elif self.compression == "bz2":
            self.file_handle = bz2.open(self.path, 'rt', encoding=self.encoding)
        else:
            self.file_handle = open(self.path, 'r', encoding=self.encoding)
    
    async def close(self):
        """Close the file handle."""
        if self.file_handle:
            self.file_handle.close()
            self.file_handle = None
    
    async def estimate_total(self) -> Optional[int]:
        """Estimate total number of articles."""
        if self.compression != "none":
            # For compressed files, rough estimate
            return self.total_size // 3000  # ~3KB per article average
        
        # For uncompressed files, sample to estimate
        try:
            with open(self.path, 'r', encoding=self.encoding) as f:
                # Read first 1MB to estimate article density
                sample_size = min(1024 * 1024, self.total_size)
                sample = f.read(sample_size)
                
                # Count triple newlines (article separators)
                separators = len(re.findall(r'\n\n\n+', sample))
                if separators > 0:
                    density = separators / sample_size
                    return int(self.total_size * density)
                else:
                    return self.total_size // 2000  # Fallback estimate
        except Exception:
            return None
    
    def get_position(self) -> int:
        """Get current position in stream."""
        return self.article_count
    
    async def __aiter__(self):
        """Async iterator interface."""
        return self
    
    async def __anext__(self):
        """Get next article."""
        result = await self.next()
        if result is None:
            raise StopAsyncIteration
        return result
    
    async def next(self) -> Optional[Dict[str, Any]]:
        """Get next Wikipedia article."""
        if not self.file_handle:
            await self.open()
        
        try:
            # Read articles separated by triple newlines
            while True:
                # Read chunk and append to buffer
                chunk = self.file_handle.read(8192)  # 8KB chunks
                if not chunk:
                    # End of file - process remaining buffer
                    if self.buffer.strip():
                        article = self._process_article(self.buffer.strip())
                        self.buffer = ""
                        if article:
                            return article
                    return None  # No more articles
                
                self.buffer += chunk
                
                # Look for article boundaries
                articles = re.split(r'\n\n\n+', self.buffer)
                
                # Keep the last incomplete article in buffer
                self.buffer = articles[-1]
                
                # Process complete articles
                for article_text in articles[:-1]:
                    article_text = article_text.strip()
                    if len(article_text) >= self.min_length:
                        article = self._process_article(article_text)
                        if article:
                            return article
                        
        except Exception as e:
            raise Exception(f"Error reading Wikipedia file: {e}")
    
    def _process_article(self, article_text: str) -> Optional[Dict[str, Any]]:
        """Process a single Wikipedia article."""
        if len(article_text) < self.min_length:
            return None
            
        self.article_count += 1
        
        # Extract title (first line)
        lines = article_text.split('\n', 1)
        if len(lines) >= 2:
            title = lines[0].strip()
            content = lines[1].strip()
        else:
            title = f"Article {self.article_count}"
            content = article_text
        
        # Split long articles
        if len(article_text) > self.max_length:
            return self._split_long_article(title, content)
        
        # Infer category from content
        category = self._infer_category(content)
        
        return {
            "content": article_text,
            "metadata": {
                "title": title,
                "article_index": self.article_count,
                "category": category,
                "content_length": len(content),
                "is_split": False
            },
            "embedding": None,
            "source_id": f"wikipedia_article_{self.article_count}",
            "item_type": "article"
        }
    
    def _split_long_article(self, title: str, content: str) -> Dict[str, Any]:
        """Split long article into manageable chunks."""
        # Split by paragraphs
        paragraphs = re.split(r'\n\s*\n', content)
        
        # Create chunk with first few paragraphs
        chunk_content = title + "\n\n"
        for paragraph in paragraphs:
            if len(chunk_content + paragraph) > self.max_length:
                break
            chunk_content += paragraph + "\n\n"
        
        category = self._infer_category(chunk_content)
        
        return {
            "content": chunk_content.strip(),
            "metadata": {
                "title": title,
                "article_index": self.article_count,
                "category": category,
                "content_length": len(chunk_content),
                "is_split": True,
                "chunk_index": 0  # First chunk
            },
            "embedding": None,
            "source_id": f"wikipedia_article_{self.article_count}_chunk_0",
            "item_type": "article_chunk"
        }
    
    def _infer_category(self, content: str) -> str:
        """Infer article category from content."""
        content_lower = content.lower()
        
        # Simple keyword-based categorization
        if any(word in content_lower for word in ['born', 'died', 'biography', 'life']):
            return 'biography'
        elif any(word in content_lower for word in ['war', 'battle', 'century', 'historical']):
            return 'history'
        elif any(word in content_lower for word in ['science', 'research', 'theory', 'discovery']):
            return 'science'
        elif any(word in content_lower for word in ['country', 'city', 'population', 'geography']):
            return 'geography'
        elif any(word in content_lower for word in ['art', 'music', 'culture', 'literature']):
            return 'arts'
        else:
            return 'general'


# Example usage for testing
async def main():
    """Test the Wikipedia adapter."""
    adapter = WikipediaAdapter()
    
    config = {
        "path": "/path/to/wikipedia.txt",
        "min_article_length": 200,
        "max_article_length": 5000
    }
    
    try:
        stream = await adapter.create_stream(json.dumps(config))
        
        # Process first 10 articles
        async with stream:
            for i in range(10):
                article = await stream.next()
                if article is None:
                    break
                    
                print(f"Article {i+1}: {article['metadata']['title']}")
                print(f"  Category: {article['metadata']['category']}")
                print(f"  Length: {len(article['content'])} chars")
                print()
                
    except Exception as e:
        print(f"Error: {e}")


if __name__ == "__main__":
    asyncio.run(main())