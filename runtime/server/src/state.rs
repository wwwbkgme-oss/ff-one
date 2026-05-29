use agents::AgentManager;
use consensus::ConsensusStore;
use drivers::{load_free_drivers, ClaudeDriver, OpenAiDriver};
use economy::EconomyStore;
use quests::QuestStore;
use sandbox::ProcessSandbox;
use security::StaticAnalyser;
use world::VoxelSimulator;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use types::world::WorldState;

pub struct AppState {
    pub world:     Arc<RwLock<WorldState>>,
    pub simulator: Arc<RwLock<VoxelSimulator>>,
    pub agents:    Arc<AgentManager>,
    pub sandbox:   Arc<ProcessSandbox>,
    pub security:  Arc<StaticAnalyser>,
    pub economy:   Arc<EconomyStore>,
    pub quests:    Arc<QuestStore>,
    pub consensus: Arc<ConsensusStore>,
}

impl AppState {
    pub fn new(seed: u64) -> Self {
        let mut manager = AgentManager::new();

        // ── Paid / cloud drivers (registered when API key present) ─────────
        if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            info!(driver = "claude", "registering driver");
            manager.register_driver(Arc::new(ClaudeDriver::new(key)));
        }
        if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            info!(driver = "opencode", "registering driver");
            manager.register_driver(Arc::new(OpenAiDriver::opencode(key)));
        }

        // ── Free-tier drivers (auto-discovered from env vars) ──────────────
        // Reads GROQ_API_KEY, CEREBRAS_API_KEY, SAMBANOVA_API_KEY,
        // OPENROUTER_API_KEY and always adds a local Ollama driver.
        for d in load_free_drivers() {
            manager.register_driver(d);
        }

        // ── Mock driver (always available for local dev / tests) ───────────
        manager.register_driver(Arc::new(drivers::MockDriver::new(
            "Mock response — set ANTHROPIC_API_KEY or a free-provider env var for a real driver.",
        )));

        Self {
            world:     Arc::new(RwLock::new(WorldState::new(seed))),
            simulator: Arc::new(RwLock::new(VoxelSimulator::new(seed))),
            agents:    Arc::new(manager),
            sandbox:   Arc::new(ProcessSandbox::new()),
            security:  Arc::new(StaticAnalyser::new()),
            economy:   Arc::new(EconomyStore::new()),
            quests:    Arc::new(QuestStore::new()),
            consensus: Arc::new(ConsensusStore::new()),
        }
    }
}
