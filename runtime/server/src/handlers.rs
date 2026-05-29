use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use contracts::traits::{
    EconomyEngine, QuestManager, SandboxExecutor, SecurityAnalyser, WorldSimulator,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use types::{
    agent::AgentKind,
    position::Position3D,
    sandbox::{CodeLanguage, SandboxConfig},
};
use uuid::Uuid;

fn err(e: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

#[derive(Serialize)]
pub struct WorldStatus {
    pub tick: u64,
    pub seed: u64,
    pub epoch: u64,
    pub chunks_loaded: usize,
    pub state_hash: String,
}
pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status":"ok","service":"forgefabrik"}))
}
pub async fn get_world(State(s): State<Arc<AppState>>) -> Json<WorldStatus> {
    let w = s.world.read().await;
    Json(WorldStatus {
        tick: w.tick,
        seed: w.seed,
        epoch: w.epoch.number,
        chunks_loaded: w.chunks.len(),
        state_hash: w.state_hash.clone(),
    })
}
pub async fn tick_world(
    State(s): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut w = s.world.write().await;
    let mut sim = s.simulator.write().await;
    let evs = sim.tick(&mut w).await.map_err(err)?;
    w.state_hash = sim.compute_hash(&w).0;
    Ok(Json(serde_json::json!({"tick":w.tick,"events":evs.len()})))
}

#[derive(Deserialize)]
pub struct SpawnReq {
    pub name: String,
    pub kind: Option<String>,
    pub x: Option<i64>,
    pub y: Option<i64>,
    pub z: Option<i64>,
}
#[derive(Serialize)]
pub struct AgentResp {
    pub id: Uuid,
    pub name: String,
    pub kind: String,
    pub position: [i64; 3],
    pub level: u64,
}
pub async fn spawn_agent(
    State(s): State<Arc<AppState>>,
    Json(req): Json<SpawnReq>,
) -> Result<Json<AgentResp>, (StatusCode, String)> {
    let kind = match req.kind.as_deref() {
        Some("opencode") => AgentKind::OpenCode,
        Some("codex") => AgentKind::Codex,
        Some("amp") => AgentKind::Amp,
        Some("pi") => AgentKind::Pi,
        Some("cursor") => AgentKind::Cursor,
        // Free-tier providers
        Some("groq") => AgentKind::Groq,
        Some("sambanova") => AgentKind::SambaNova,
        Some("ollama") => AgentKind::Ollama,
        Some("openrouter") => AgentKind::OpenRouter,
        Some("cerebras") => AgentKind::Cerebras,
        _ => AgentKind::Claude,
    };
    let pos = Position3D::new(req.x.unwrap_or(0), req.y.unwrap_or(64), req.z.unwrap_or(0));
    let a = s.agents.spawn(req.name, kind, pos).await.map_err(err)?;
    Ok(Json(AgentResp {
        id: a.id,
        name: a.name.clone(),
        kind: a.kind.to_string(),
        position: [a.position.x, a.position.y, a.position.z],
        level: a.level(),
    }))
}
pub async fn list_agents(State(s): State<Arc<AppState>>) -> Json<Vec<AgentResp>> {
    let agents = s.agents.list().await;
    Json(
        agents
            .iter()
            .map(|a| AgentResp {
                id: a.id,
                name: a.name.clone(),
                kind: a.kind.to_string(),
                position: [a.position.x, a.position.y, a.position.z],
                level: a.level(),
            })
            .collect(),
    )
}

#[derive(Deserialize)]
pub struct CmdReq {
    pub command: String,
}
pub async fn command_agent(
    State(s): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<CmdReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let w = s.world.read().await;
    let resp = s.agents.command(id, &req.command, &w).await.map_err(err)?;
    Ok(Json(serde_json::json!({"response":resp})))
}

#[derive(Deserialize)]
pub struct CreateSbReq {
    pub agent_id: Uuid,
    pub language: Option<String>,
}
pub async fn create_sandbox(
    State(s): State<Arc<AppState>>,
    Json(req): Json<CreateSbReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let lang = match req.language.as_deref() {
        Some("javascript") => CodeLanguage::JavaScript,
        Some("bash") => CodeLanguage::Bash,
        Some("lua") => CodeLanguage::Lua,
        _ => CodeLanguage::Python,
    };
    let id = s
        .sandbox
        .create(SandboxConfig::new(req.agent_id, lang))
        .await
        .map_err(err)?;
    Ok(Json(serde_json::json!({"sandbox_id":id})))
}

#[derive(Deserialize)]
pub struct ExecReq {
    pub code: String,
}
pub async fn execute_code(
    State(s): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<ExecReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let a = s
        .security
        .assess(&req.code, "auto", Uuid::nil())
        .await
        .map_err(err)?;
    if !a.is_safe() {
        return Err((StatusCode::FORBIDDEN, format!("{:?}", a.decision)));
    }
    let r = s.sandbox.execute(id, &req.code).await.map_err(err)?;
    Ok(Json(
        serde_json::json!({"exit_status":format!("{:?}",r.exit_status),"stdout":r.stdout,"stderr":r.stderr,"duration_ms":r.duration_ms}),
    ))
}

#[derive(Deserialize)]
pub struct AssessReq {
    pub code: String,
    pub language: String,
    pub agent_id: Option<Uuid>,
}
pub async fn assess_code(
    State(s): State<Arc<AppState>>,
    Json(req): Json<AssessReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let a = s
        .security
        .assess(
            &req.code,
            &req.language,
            req.agent_id.unwrap_or_else(Uuid::nil),
        )
        .await
        .map_err(err)?;
    Ok(Json(
        serde_json::json!({"safe":a.is_safe(),"risk_score":a.risk_score,"decision":format!("{:?}",a.decision),"findings":a.findings.len()}),
    ))
}

pub async fn list_quests(
    State(s): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let qs = s.quests.available_quests().await.map_err(err)?;
    Ok(Json(serde_json::json!({"count":qs.len(),"quests":qs})))
}

#[derive(Deserialize)]
pub struct AcceptReq {
    pub agent_id: Uuid,
}
pub async fn accept_quest(
    State(s): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<AcceptReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    s.quests.accept_quest(id, req.agent_id).await.map_err(err)?;
    Ok(Json(serde_json::json!({"accepted":true})))
}

pub async fn get_market(
    State(s): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let ls = s.economy.get_listings().await.map_err(err)?;
    let as_ = s.economy.get_auctions().await.map_err(err)?;
    Ok(Json(
        serde_json::json!({"listings":ls.len(),"auctions":as_.len()}),
    ))
}

/// GET /drivers — list all registered AgentDrivers.
///
/// Returns the sorted names of every driver registered at startup.
/// Use the `kind` field when spawning an agent to select a driver.
pub async fn list_drivers(State(s): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let names = s.agents.driver_names();
    Json(serde_json::json!({"count": names.len(), "drivers": names}))
}
