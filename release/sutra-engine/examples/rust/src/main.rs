use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use serde::{Serialize, Deserialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug)]
struct LearnRequest {
    #[serde(rename = "LearnConceptV2")]
    learn_concept_v2: LearnConceptV2,
}

#[derive(Serialize, Deserialize, Debug)]
struct LearnConceptV2 {
    content: String,
    options: LearnOptions,
}

#[derive(Serialize, Deserialize, Debug)]
struct LearnOptions {
    generate_embedding: bool,
    embedding_model: Option<String>,
    extract_associations: bool,
    min_association_confidence: f32,
    max_associations_per_concept: usize,
    strength: f32,
    confidence: f32,
}

/**
 * Rust Example for Sutra Engine (TCP)
 * Uses tokio for async I/O and rmp-serde for MessagePack.
 */
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051";
    let mut stream = TcpStream::connect(addr).await?;
    println!("Connected to Sutra Engine at {}", addr);

    // 1. Prepare Request
    let request = LearnRequest {
        learn_concept_v2: LearnConceptV2 {
            content: "Rust is a systems programming language focused on safety and performance.".to_string(),
            options: LearnOptions {
                generate_embedding: false,
                embedding_model: None,
                extract_associations: true,
                min_association_confidence: 0.5,
                max_associations_per_concept: 10,
                strength: 1.0,
                confidence: 1.0,
            },
        },
    };

    // 2. Serialize to MessagePack
    let payload = rmp_serde::to_vec_named(&request)?;
    
    // 3. Send Length Header (4-byte big-endian)
    let len = payload.len() as u32;
    stream.write_u32(len).await?;
    
    // 4. Send Payload
    stream.write_all(&payload).await?;
    println!("Request sent");

    // 5. Read Response Length
    let resp_len = stream.read_u32().await?;
    let mut resp_buf = vec![0u8; resp_len as usize];
    
    // 6. Read Response Payload
    stream.read_exact(&mut resp_buf).await?;
    
    // 7. Deserialize
    let response: serde_json::Value = rmp_serde::from_slice(&resp_buf)?;
    println!("Response received: {:#?}", response);

    Ok(())
}
