use agents::AgentManager;
use consensus::ConsensusStore;
use economy::EconomyStore;
use quests::QuestStore;
use sandbox::ProcessSandbox;
use security::StaticAnalyser;
use world::VoxelSimulator;
use std::sync::Arc;
use tokio::sync::RwLock;
use types::world::WorldState;
pub struct AppState{
    pub world:Arc<RwLock<WorldState>>,
    pub simulator:Arc<RwLock<VoxelSimulator>>,
    pub agents:Arc<AgentManager>,
    pub sandbox:Arc<ProcessSandbox>,
    pub security:Arc<StaticAnalyser>,
    pub economy:Arc<EconomyStore>,
    pub quests:Arc<QuestStore>,
    pub consensus:Arc<ConsensusStore>,
}
impl AppState{
    pub fn new(seed:u64)->Self{Self{
        world:Arc::new(RwLock::new(WorldState::new(seed))),
        simulator:Arc::new(RwLock::new(VoxelSimulator::new(seed))),
        agents:Arc::new(AgentManager::new()),
        sandbox:Arc::new(ProcessSandbox::new()),
        security:Arc::new(StaticAnalyser::new()),
        economy:Arc::new(EconomyStore::new()),
        quests:Arc::new(QuestStore::new()),
        consensus:Arc::new(ConsensusStore::new()),
    }}
}
