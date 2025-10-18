use std::collections::HashMap;
use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::RwLock;
use tokio::net::{TcpListener, TcpStream};
use chrono::{Utc};
use sutra_grid_events::{EventEmitter, GridEvent};
use sutra_protocol::{GridMessage, GridResponse, AgentRecord, recv_message, send_message};

mod binary_server;

// ===== Data Structures =====

/// Master's internal agent record with additional tracking fields
#[derive(Debug, Clone)]
struct MasterAgentRecord {
    agent_id: String,
    hostname: String,
    platform: String,
    agent_endpoint: String,  // "hostname:port" for Agent TCP server
    max_storage_nodes: u32,
    current_storage_nodes: u32,
    last_heartbeat: u64,
    status: AgentStatus,
    storage_nodes: Vec<StorageNodeRecord>,
}

impl MasterAgentRecord {
    /// Convert to protocol AgentRecord for transmission
    fn to_protocol(&self) -> AgentRecord {
        AgentRecord {
            agent_id: self.agent_id.clone(),
            hostname: self.hostname.clone(),
            platform: self.platform.clone(),
            status: self.status.as_str().to_string(),
            max_storage_nodes: self.max_storage_nodes,
            current_storage_nodes: self.current_storage_nodes,
            last_heartbeat: self.last_heartbeat,
        }
    }
}

#[derive(Debug, Clone)]
struct StorageNodeRecord {
    node_id: String,
    endpoint: String,
    pid: u32,
    status: NodeStatus,
}

#[derive(Debug, Clone, PartialEq)]
enum AgentStatus {
    Healthy,
    Degraded,
    Offline,
}

impl AgentStatus {
    fn as_str(&self) -> &'static str {
        match self {
            AgentStatus::Healthy => "healthy",
            AgentStatus::Degraded => "degraded",
            AgentStatus::Offline => "offline",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum NodeStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
}

impl NodeStatus {
    fn as_str(&self) -> &'static str {
        match self {
            NodeStatus::Starting => "starting",
            NodeStatus::Running => "running",
            NodeStatus::Stopping => "stopping",
            NodeStatus::Stopped => "stopped",
            NodeStatus::Failed => "failed",
        }
    }
}

// ===== Grid Master Service =====

#[derive(Clone)]
struct GridMasterService {
    agents: Arc<RwLock<HashMap<String, MasterAgentRecord>>>,
    events: Option<EventEmitter>,
}

impl GridMasterService {
    fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            events: None,
        }
    }
    
    fn new_with_events(events: EventEmitter) -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            events: Some(events),
        }
    }
    
    /// Check for stale agents (no heartbeat in 30 seconds)
    async fn check_agent_health(&self) {
        let mut agents = self.agents.write().await;
        let now = current_timestamp();
        
        for agent in agents.values_mut() {
            let seconds_since_heartbeat = now.saturating_sub(agent.last_heartbeat);
            
            if seconds_since_heartbeat > 30 {
                if agent.status != AgentStatus::Offline {
                    log::warn!("❌ Agent {} is offline (no heartbeat for {}s)", 
                              agent.agent_id, seconds_since_heartbeat);
                    
                    // Emit offline event
                    if let Some(events) = &self.events {
                        events.emit(GridEvent::AgentOffline {
                            agent_id: agent.agent_id.clone(),
                            last_seen: Utc::now(),
                            timestamp: Utc::now(),
                        });
                    }
                    
                    agent.status = AgentStatus::Offline;
                }
            } else if seconds_since_heartbeat > 15 {
                if agent.status != AgentStatus::Degraded {
                    log::warn!("⚠️  Agent {} is degraded (no heartbeat for {}s)", 
                              agent.agent_id, seconds_since_heartbeat);
                    
                    // Emit degraded event
                    if let Some(events) = &self.events {
                        events.emit(GridEvent::AgentDegraded {
                            agent_id: agent.agent_id.clone(),
                            seconds_since_heartbeat,
                            timestamp: Utc::now(),
                        });
                    }
                    
                    agent.status = AgentStatus::Degraded;
                }
            }
        }
    }
    
    /// Get cluster status
    async fn get_cluster_status_internal(&self) -> (u32, u32, u32, u32, &'static str) {
        let agents = self.agents.read().await;
        
        let total_agents = agents.len() as u32;
        let healthy_agents = agents.values()
            .filter(|a| a.status == AgentStatus::Healthy)
            .count() as u32;
        
        let total_storage_nodes: u32 = agents.values()
            .map(|a| a.current_storage_nodes)
            .sum();
        
        let running_storage_nodes: u32 = agents.values()
            .flat_map(|a| &a.storage_nodes)
            .filter(|n| n.status == NodeStatus::Running)
            .count() as u32;
        
        let status = if healthy_agents == 0 && total_agents > 0 {
            "critical"
        } else if healthy_agents < total_agents / 2 && total_agents > 0 {
            "degraded"
        } else {
            "healthy"
        };
        
        (total_agents, healthy_agents, total_storage_nodes, running_storage_nodes, status)
    }
}

// ===== TCP server =====
async fn handle_client(mut stream: TcpStream, state: GridMasterService) -> std::io::Result<()> {
    loop {
        let msg: GridMessage = match recv_message(&mut stream).await {
            Ok(m) => m,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e),
        };

        let response = match msg {
            GridMessage::RegisterAgent { agent_id, hostname, platform, max_storage_nodes, version:_, agent_endpoint } => {
                let record = MasterAgentRecord {
                    agent_id: agent_id.clone(),
                    hostname,
                    platform,
                    agent_endpoint,
                    max_storage_nodes,
                    current_storage_nodes: 0,
                    last_heartbeat: current_timestamp(),
                    status: AgentStatus::Healthy,
                    storage_nodes: Vec::new(),
                };
                state.agents.write().await.insert(agent_id.clone(), record);
                GridResponse::RegisterAgentOk { success: true, master_version: "1.0.0".into(), error_message: None }
            }
            GridMessage::Heartbeat { agent_id, storage_node_count, timestamp:_ } => {
                if let Some(a) = state.agents.write().await.get_mut(&agent_id) {
                    a.current_storage_nodes = storage_node_count;
                    a.last_heartbeat = current_timestamp();
                }
                GridResponse::HeartbeatOk { acknowledged: true, timestamp: current_timestamp() }
            }
            GridMessage::ListAgents => {
                let agents = state.agents.read().await
                    .values()
                    .map(|a| a.to_protocol())
                    .collect();
                GridResponse::ListAgentsOk { agents }
            }
            GridMessage::GetClusterStatus => {
                let agents = state.agents.read().await;
                let total_agents = agents.len() as u32;
                let healthy_agents = agents.values().filter(|a| a.status==AgentStatus::Healthy).count() as u32;
                let total_storage_nodes = agents.values().map(|a| a.current_storage_nodes).sum();
                let running_storage_nodes = total_storage_nodes;
                GridResponse::GetClusterStatusOk { total_agents, healthy_agents, total_storage_nodes, running_storage_nodes, status: "ok".into() }
            }
            _ => GridResponse::Error { message: "Unsupported operation".into() },
        };

        send_message(&mut stream, &response).await?;
    }
    Ok(())
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    // Create Grid master state
    let state = GridMasterService::new();

    // Start TCP server
    let tcp_port: u16 = std::env::var("GRID_MASTER_TCP_PORT").unwrap_or("7002".into()).parse().unwrap_or(7002);
    let tcp_addr = format!("0.0.0.0:{}", tcp_port);
    let listener = TcpListener::bind(&tcp_addr).await?;
    log::info!("Grid Master TCP listening on {}", tcp_addr);

    // Start HTTP binary server
    let http_port: u16 = std::env::var("GRID_MASTER_HTTP_PORT").unwrap_or("7001".into()).parse().unwrap_or(7001);
    let binaries_dir = PathBuf::from("/data/binaries");
    let distribution = binary_server::BinaryDistribution::new(binaries_dir.clone());
    let app = binary_server::create_router(distribution, binaries_dir);
    let http_addr: std::net::SocketAddr = format!("0.0.0.0:{}", http_port).parse().unwrap();
    let http_server = axum::serve(tokio::net::TcpListener::bind(http_addr).await?, app);
    log::info!("Grid Master HTTP listening on {}", http_port);

    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    let st = state.clone();
                    log::info!("Client connected: {}", addr);
                    tokio::spawn(async move {
                        if let Err(e) = handle_client(stream, st).await {
                            log::error!("Client error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    log::error!("Accept error: {}", e);
                }
            }
        }
    });

    http_server.await?;
    Ok(())
}

