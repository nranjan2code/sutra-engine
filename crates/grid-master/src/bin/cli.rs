#!/usr/bin/env rust

use clap::{Parser, Subcommand};
use sutra_protocol::{GridMessage, GridResponse};
use tokio::net::TcpStream;
use anyhow::Result;

#[derive(Parser)]
#[command(name = "sutra-grid-cli")]
#[command(about = "CLI for Sutra Grid Master", long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "localhost:7002")]
    master: String,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all registered agents
    ListAgents,
    
    /// Get cluster status
    Status,
    
    /// Spawn a storage node on an agent
    Spawn {
        /// Agent ID to spawn the node on
        #[arg(short, long)]
        agent: String,
        
        /// Port for the storage node
        #[arg(short, long)]
        port: u32,
        
        /// Storage path for the node
        #[arg(short, long, default_value = "/tmp/storage")]
        storage_path: String,
        
        /// Memory limit in MB
        #[arg(short, long, default_value = "512")]
        memory: u64,
    },
    
    /// Stop a storage node
    Stop {
        /// Node ID to stop
        #[arg(short, long)]
        node: String,
        
        /// Agent ID owning the node
        #[arg(short, long)]
        agent: String,
    },
    
    /// Get status of a specific storage node
    NodeStatus {
        /// Node ID to query
        #[arg(short, long)]
        node: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Connect to Grid Master via TCP
    let mut stream = TcpStream::connect(&cli.master).await?;
    
    match cli.command {
        Commands::ListAgents => {
            // Send request
            sutra_protocol::send_message(&mut stream, &GridMessage::ListAgents).await?;
            
            // Receive response
            let response: GridResponse = sutra_protocol::recv_message(&mut stream).await?;
            
            match response {
                GridResponse::ListAgentsOk { agents } => {
                    println!("📋 Registered Agents ({}):", agents.len());
                    println!();
                    
                    for agent in agents {
                        println!("🖥️  Agent: {}", agent.agent_id);
                        println!("   Hostname: {}", agent.hostname);
                        println!("   Platform: {}", agent.platform);
                        println!("   Status: {}", agent.status);
                        println!("   Storage Nodes: {}/{}", agent.current_storage_nodes, agent.max_storage_nodes);
                        println!("   Last Heartbeat: {} seconds ago", 
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)?
                                .as_secs()
                                .saturating_sub(agent.last_heartbeat)
                        );
                        println!();
                    }
                }
                GridResponse::Error { message } => {
                    eprintln!("❌ Error: {}", message);
                }
                _ => {
                    eprintln!("❌ Unexpected response type");
                }
            }
        }
        
        Commands::Status => {
            // Send request
            sutra_protocol::send_message(&mut stream, &GridMessage::GetClusterStatus).await?;
            
            // Receive response
            let response: GridResponse = sutra_protocol::recv_message(&mut stream).await?;
            
            match response {
                GridResponse::GetClusterStatusOk { total_agents, healthy_agents, total_storage_nodes, running_storage_nodes, status } => {
                    println!("📊 Cluster Status");
                    println!("================");
                    println!("Total Agents: {}", total_agents);
                    println!("Healthy Agents: {}", healthy_agents);
                    println!("Total Storage Nodes: {}", total_storage_nodes);
                    println!("Running Storage Nodes: {}", running_storage_nodes);
                    println!("Overall Status: {}", status);
                }
                GridResponse::Error { message } => {
                    eprintln!("❌ Error: {}", message);
                }
                _ => {
                    eprintln!("❌ Unexpected response type");
                }
            }
        }
        
        Commands::Spawn { agent, port, storage_path, memory } => {
            println!("📦 Spawning storage node on agent {}...", agent);
            
            // Send request
            sutra_protocol::send_message(&mut stream, &GridMessage::SpawnStorageNode {
                agent_id: agent.clone(),
                storage_path,
                memory_limit_mb: memory,
                port,
            }).await?;
            
            // Receive response
            let response: GridResponse = sutra_protocol::recv_message(&mut stream).await?;
            
            match response {
                GridResponse::SpawnStorageNodeOk { node_id, endpoint, success, error_message } => {
                    if success {
                        println!("✅ Storage node spawned successfully!");
                        println!("   Node ID: {}", node_id);
                        println!("   Endpoint: {}", endpoint);
                    } else {
                        println!("❌ Failed to spawn storage node");
                        if let Some(err) = error_message {
                            println!("   Error: {}", err);
                        }
                    }
                }
                GridResponse::Error { message } => {
                    eprintln!("❌ Error: {}", message);
                }
                _ => {
                    eprintln!("❌ Unexpected response type");
                }
            }
        }
        
        Commands::Stop { node, agent } => {
            println!("🛑 Stopping storage node {}...", node);
            
            // Send request
            sutra_protocol::send_message(&mut stream, &GridMessage::StopStorageNode {
                agent_id: agent,
                node_id: node.clone(),
            }).await?;
            
            // Receive response
            let response: GridResponse = sutra_protocol::recv_message(&mut stream).await?;
            
            match response {
                GridResponse::StopStorageNodeOk { success, error_message } => {
                    if success {
                        println!("✅ Storage node stopped successfully!");
                    } else {
                        println!("❌ Failed to stop storage node");
                        if let Some(err) = error_message {
                            println!("   Error: {}", err);
                        }
                    }
                }
                GridResponse::Error { message } => {
                    eprintln!("❌ Error: {}", message);
                }
                _ => {
                    eprintln!("❌ Unexpected response type");
                }
            }
        }
        
        Commands::NodeStatus { node } => {
            // Send request
            sutra_protocol::send_message(&mut stream, &GridMessage::GetStorageNodeStatus {
                node_id: node.clone(),
            }).await?;
            
            // Receive response
            let response: GridResponse = sutra_protocol::recv_message(&mut stream).await?;
            
            match response {
                GridResponse::GetStorageNodeStatusOk { node_id, status, pid, endpoint } => {
                    println!("📦 Storage Node: {}", node_id);
                    println!("   Status: {}", status);
                    println!("   PID: {}", pid);
                    println!("   Endpoint: {}", endpoint);
                }
                GridResponse::Error { message } => {
                    eprintln!("❌ Error: {}", message);
                }
                _ => {
                    eprintln!("❌ Unexpected response type");
                }
            }
        }
    }
    
    Ok(())
}
