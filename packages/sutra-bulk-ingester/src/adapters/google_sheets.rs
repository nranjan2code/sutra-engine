//! Google Sheets adapter for financial data ingestion
//! 
//! Supports GOOGLEFINANCE function data and historical stock information
//! Built for high-performance bulk ingestion of financial time series

use super::{DataItem, DataStream, IngestionAdapter, AdapterInfo};
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value as JsonValue, json};
use std::collections::HashMap;
use tracing::{info, warn, debug};
use chrono::{NaiveDate, Datelike};

/// Google Sheets API configuration
#[derive(Debug, Clone, Deserialize)]
pub struct GoogleSheetsConfig {
    /// Spreadsheet ID (from URL)
    pub spreadsheet_id: String,
    
    /// Sheet name or ID
    pub sheet_name: String,
    
    /// Range to read (A1 notation, e.g., "A1:Z1000")
    pub range: String,
    
    /// Google API Key (for read-only access)
    pub api_key: String,
    
    /// Data format interpretation
    pub format: GoogleSheetsFormat,
    
    /// Optional value filters
    pub filters: Option<Vec<DataFilter>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoogleSheetsFormat {
    /// GOOGLEFINANCE function output (ticker, date, price, volume, etc.)
    GoogleFinance,
    
    /// Custom financial time series
    FinancialTimeSeries,
    
    /// Generic tabular data
    Tabular,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DataFilter {
    pub column: String,
    pub operation: FilterOperation,
    pub value: JsonValue,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOperation {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    Contains,
    NotEmpty,
}

/// Google Sheets API response structures
#[derive(Debug, Deserialize)]
struct SheetsApiResponse {
    values: Option<Vec<Vec<String>>>,
}

#[derive(Debug, Deserialize)]
struct SpreadsheetMetadata {
    sheets: Vec<SheetMetadata>,
}

#[derive(Debug, Deserialize)]
struct SheetMetadata {
    properties: SheetProperties,
}

#[derive(Debug, Deserialize)]
struct SheetProperties {
    title: String,
    #[serde(rename = "sheetId")]
    sheet_id: i32,
    #[serde(rename = "gridProperties")]
    grid_properties: GridProperties,
}

#[derive(Debug, Deserialize)]
struct GridProperties {
    #[serde(rename = "rowCount")]
    row_count: i32,
    #[serde(rename = "columnCount")]
    column_count: i32,
}

/// Financial data point from Google Sheets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialDataPoint {
    pub ticker: String,
    pub date: String,
    pub price: Option<f64>,
    pub volume: Option<i64>,
    pub market_cap: Option<f64>,
    pub pe_ratio: Option<f64>,
    pub dividend_yield: Option<f64>,
    pub beta: Option<f64>,
    pub data_type: String, // "price", "volume", "financials", etc.
    pub source_row: u64,
}

impl FinancialDataPoint {
    /// Convert to semantic content for Sutra learning
    pub fn to_semantic_content(&self) -> String {
        let mut content_parts = vec![
            format!("Stock ticker: {}", self.ticker),
            format!("Date: {}", self.date),
            format!("Data type: {}", self.data_type),
        ];
        
        if let Some(price) = self.price {
            content_parts.push(format!("Stock price: ${:.2}", price));
        }
        
        if let Some(volume) = self.volume {
            content_parts.push(format!("Trading volume: {} shares", volume));
        }
        
        if let Some(market_cap) = self.market_cap {
            content_parts.push(format!("Market capitalization: ${:.0}", market_cap));
        }
        
        if let Some(pe) = self.pe_ratio {
            content_parts.push(format!("P/E ratio: {:.2}", pe));
        }
        
        if let Some(div_yield) = self.dividend_yield {
            content_parts.push(format!("Dividend yield: {:.2}%", div_yield * 100.0));
        }
        
        if let Some(beta) = self.beta {
            content_parts.push(format!("Beta: {:.2}", beta));
        }
        
        // Create semantic relationships for Sutra to learn
        let semantic_content = format!(
            "On {}, {} had the following financial metrics: {}. This represents {} market data for {}.",
            self.date,
            self.ticker,
            content_parts[2..].join(", "),
            self.data_type,
            self.ticker
        );
        
        semantic_content
    }
    
    /// Generate metadata for enhanced learning
    pub fn to_metadata(&self) -> HashMap<String, JsonValue> {
        let mut metadata = HashMap::new();
        
        metadata.insert("ticker".to_string(), json!(self.ticker));
        metadata.insert("date".to_string(), json!(self.date));
        metadata.insert("data_type".to_string(), json!(self.data_type));
        metadata.insert("source_row".to_string(), json!(self.source_row));
        metadata.insert("domain".to_string(), json!("finance"));
        metadata.insert("entity_type".to_string(), json!("stock_data"));
        
        if let Some(price) = self.price {
            metadata.insert("price".to_string(), json!(price));
        }
        
        if let Some(volume) = self.volume {
            metadata.insert("volume".to_string(), json!(volume));
        }
        
        if let Some(market_cap) = self.market_cap {
            metadata.insert("market_cap".to_string(), json!(market_cap));
        }
        
        // Add temporal metadata for Sutra's temporal reasoning
        if let Ok(parsed_date) = NaiveDate::parse_from_str(&self.date, "%Y-%m-%d") {
            metadata.insert("year".to_string(), json!(parsed_date.year()));
            metadata.insert("month".to_string(), json!(parsed_date.month()));
            metadata.insert("day".to_string(), json!(parsed_date.day()));
            metadata.insert("weekday".to_string(), json!(parsed_date.weekday().to_string()));
        }
        
        metadata
    }
}

/// Google Sheets adapter implementation
pub struct GoogleSheetsAdapter {
    client: Client,
}

impl GoogleSheetsAdapter {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
    
    /// Test Google Sheets API connectivity
    async fn test_connection(&self, config: &GoogleSheetsConfig) -> Result<()> {
        let url = format!(
            "https://sheets.googleapis.com/v4/spreadsheets/{}?key={}",
            config.spreadsheet_id, config.api_key
        );
        
        let response = self.client.get(&url).send().await?;
        let status = response.status();
        
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Google Sheets API error ({}): {}",
                status,
                error_text
            ));
        }
        
        Ok(())
    }
    
    /// Get sheet metadata
    async fn get_sheet_metadata(&self, config: &GoogleSheetsConfig) -> Result<GridProperties> {
        let url = format!(
            "https://sheets.googleapis.com/v4/spreadsheets/{}?key={}",
            config.spreadsheet_id, config.api_key
        );
        
        let response = self.client.get(&url).send().await?;
        let metadata: SpreadsheetMetadata = response.json().await?;
        
        for sheet in metadata.sheets {
            if sheet.properties.title == config.sheet_name {
                return Ok(sheet.properties.grid_properties);
            }
        }
        
        Err(anyhow::anyhow!("Sheet '{}' not found", config.sheet_name))
    }
}

#[async_trait]
impl IngestionAdapter for GoogleSheetsAdapter {
    fn name(&self) -> &str {
        "google_sheets"
    }
    
    fn supported_types(&self) -> Vec<&str> {
        vec!["google_sheets", "finance", "spreadsheet"]
    }
    
    async fn validate_config(&self, config: &JsonValue) -> Result<()> {
        let sheets_config: GoogleSheetsConfig = serde_json::from_value(config.clone())?;
        
        // Validate required fields
        if sheets_config.spreadsheet_id.is_empty() {
            return Err(anyhow::anyhow!("spreadsheet_id is required"));
        }
        
        if sheets_config.api_key.is_empty() {
            return Err(anyhow::anyhow!("api_key is required"));
        }
        
        // Test API connectivity
        self.test_connection(&sheets_config).await?;
        
        info!("Google Sheets configuration validated successfully");
        Ok(())
    }
    
    async fn create_stream(&self, config: &JsonValue) -> Result<Box<dyn DataStream>> {
        let sheets_config: GoogleSheetsConfig = serde_json::from_value(config.clone())?;
        
        Ok(Box::new(
            GoogleSheetsStream::new(self.client.clone(), sheets_config).await?
        ))
    }
    
    fn info(&self) -> AdapterInfo {
        AdapterInfo {
            name: "google_sheets".to_string(),
            description: "Google Sheets adapter for financial data (GOOGLEFINANCE function)".to_string(),
            version: "1.0.0".to_string(),
            supported_types: vec![
                "google_sheets".to_string(),
                "finance".to_string(),
                "spreadsheet".to_string(),
            ],
            config_schema: json!({
                "type": "object",
                "properties": {
                    "spreadsheet_id": {
                        "type": "string",
                        "description": "Google Sheets spreadsheet ID (from URL)"
                    },
                    "sheet_name": {
                        "type": "string", 
                        "description": "Sheet name or ID"
                    },
                    "range": {
                        "type": "string",
                        "description": "Range in A1 notation (e.g., 'A1:Z1000')"
                    },
                    "api_key": {
                        "type": "string",
                        "description": "Google API key for Sheets API access"
                    },
                    "format": {
                        "type": "string",
                        "enum": ["google_finance", "financial_time_series", "tabular"],
                        "default": "google_finance"
                    }
                },
                "required": ["spreadsheet_id", "sheet_name", "range", "api_key"]
            }),
        }
    }
}

/// Data stream implementation for Google Sheets
pub struct GoogleSheetsStream {
    client: Client,
    config: GoogleSheetsConfig,
    data: Vec<Vec<String>>,
    position: usize,
    total_rows: u64,
    headers: Vec<String>,
}

impl GoogleSheetsStream {
    async fn new(client: Client, config: GoogleSheetsConfig) -> Result<Self> {
        info!("Creating Google Sheets stream for {}", config.spreadsheet_id);
        
        // Fetch data from Google Sheets API
        let url = format!(
            "https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}!{}?key={}",
            config.spreadsheet_id,
            config.sheet_name,
            config.range,
            config.api_key
        );
        
        debug!("Fetching data from: {}", url);
        
        let response = client.get(&url).send().await?;
        let status = response.status();
        
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Failed to fetch sheet data ({}): {}",
                status,
                error_text
            ));
        }
        
        let api_response: SheetsApiResponse = response.json().await?;
        let data = api_response.values.unwrap_or_default();
        
        let total_rows = data.len() as u64;
        let headers = if !data.is_empty() {
            data[0].clone()
        } else {
            vec![]
        };
        
        info!(
            "Loaded {} rows from Google Sheets (sheet: {}, range: {})",
            total_rows, config.sheet_name, config.range
        );
        
        Ok(Self {
            client,
            config,
            data,
            position: 1, // Skip header row
            total_rows,
            headers,
        })
    }
    
    /// Parse row into financial data point
    fn parse_financial_row(&self, row: &[String], row_index: usize) -> Result<FinancialDataPoint> {
        // Handle different Google Finance formats
        match self.config.format {
            GoogleSheetsFormat::GoogleFinance => {
                // Expected columns: Date, Ticker, Price, Volume, Market Cap, P/E, Dividend Yield, Beta
                let ticker = row.get(1).cloned().unwrap_or_default();
                let date = row.get(0).cloned().unwrap_or_default();
                
                let price = row.get(2)
                    .and_then(|s| s.parse::<f64>().ok());
                    
                let volume = row.get(3)
                    .and_then(|s| s.replace(",", "").parse::<i64>().ok());
                    
                let market_cap = row.get(4)
                    .and_then(|s| s.replace(",", "").replace("B", "000000000").replace("M", "000000").parse::<f64>().ok());
                    
                let pe_ratio = row.get(5)
                    .and_then(|s| s.parse::<f64>().ok());
                    
                let dividend_yield = row.get(6)
                    .and_then(|s| s.replace("%", "").parse::<f64>().map(|v| v / 100.0).ok());
                    
                let beta = row.get(7)
                    .and_then(|s| s.parse::<f64>().ok());
                
                Ok(FinancialDataPoint {
                    ticker,
                    date,
                    price,
                    volume,
                    market_cap,
                    pe_ratio,
                    dividend_yield,
                    beta,
                    data_type: "market_data".to_string(),
                    source_row: row_index as u64,
                })
            }
            GoogleSheetsFormat::FinancialTimeSeries => {
                // Custom format - adapt as needed
                let ticker = row.get(0).cloned().unwrap_or_default();
                let date = row.get(1).cloned().unwrap_or_default();
                let price = row.get(2).and_then(|s| s.parse::<f64>().ok());
                
                Ok(FinancialDataPoint {
                    ticker,
                    date,
                    price,
                    volume: None,
                    market_cap: None,
                    pe_ratio: None,
                    dividend_yield: None,
                    beta: None,
                    data_type: "time_series".to_string(),
                    source_row: row_index as u64,
                })
            }
            GoogleSheetsFormat::Tabular => {
                // Generic tabular format
                let content = row.join(", ");
                Ok(FinancialDataPoint {
                    ticker: "UNKNOWN".to_string(),
                    date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                    price: None,
                    volume: None,
                    market_cap: None,
                    pe_ratio: None,
                    dividend_yield: None,
                    beta: None,
                    data_type: "tabular".to_string(),
                    source_row: row_index as u64,
                })
            }
        }
    }
}

#[async_trait]
impl DataStream for GoogleSheetsStream {
    async fn next(&mut self) -> Option<Result<DataItem>> {
        if self.position >= self.data.len() {
            return None;
        }
        
        let row = &self.data[self.position];
        let row_index = self.position;
        self.position += 1;
        
        // Skip empty rows
        if row.is_empty() || row.iter().all(|cell| cell.trim().is_empty()) {
            return self.next().await;
        }
        
        match self.parse_financial_row(row, row_index) {
            Ok(financial_data) => {
                let content = financial_data.to_semantic_content();
                let metadata = financial_data.to_metadata();
                
                Some(Ok(DataItem {
                    content,
                    metadata,
                    embedding: None,
                    source_id: format!("{}_{}", self.config.spreadsheet_id, row_index),
                    item_type: "financial_data".to_string(),
                }))
            }
            Err(e) => {
                warn!("Failed to parse row {}: {}", row_index, e);
                // Continue with next row instead of failing
                self.next().await
            }
        }
    }
    
    async fn estimate_total(&self) -> Result<Option<u64>> {
        // We already loaded all data, so we know the exact count
        Ok(Some(self.total_rows.saturating_sub(1))) // Subtract header row
    }
    
    fn position(&self) -> u64 {
        self.position as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_financial_data_point_to_content() {
        let data_point = FinancialDataPoint {
            ticker: "NVDA".to_string(),
            date: "2025-11-07".to_string(),
            price: Some(142.50),
            volume: Some(45_000_000),
            market_cap: Some(3_500_000_000_000.0),
            pe_ratio: Some(68.5),
            dividend_yield: Some(0.002),
            beta: Some(1.45),
            data_type: "market_data".to_string(),
            source_row: 1,
        };
        
        let content = data_point.to_semantic_content();
        assert!(content.contains("NVDA"));
        assert!(content.contains("2025-11-07"));
        assert!(content.contains("$142.50"));
        assert!(content.contains("45000000 shares"));
    }
    
    #[test]
    fn test_financial_data_point_metadata() {
        let data_point = FinancialDataPoint {
            ticker: "GOOGL".to_string(),
            date: "2025-11-07".to_string(),
            price: Some(175.25),
            volume: None,
            market_cap: None,
            pe_ratio: None,
            dividend_yield: None,
            beta: None,
            data_type: "price".to_string(),
            source_row: 42,
        };
        
        let metadata = data_point.to_metadata();
        assert_eq!(metadata.get("ticker").unwrap(), &json!("GOOGL"));
        assert_eq!(metadata.get("price").unwrap(), &json!(175.25));
        assert_eq!(metadata.get("domain").unwrap(), &json!("finance"));
    }
}