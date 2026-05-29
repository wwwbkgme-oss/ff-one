
use async_trait::async_trait;
use contracts::{error::{FfError, Result}, traits::QuestManager};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use types::{
    agent::Agent,
    quest::{Quest, QuestObjective, QuestReward, QuestStatus},
    world::WorldState,
};
use uuid::Uuid;

pub struct QuestStore {
    quests: Arc<RwLock<HashMap<Uuid, Quest>>>,
}

impl QuestStore {
    pub fn new() -> Self { Self { quests: Arc::new(RwLock::new(HashMap::new())) } }

    /// Fügt einen Quest manuell hinzu (z. B. aus Konfiguration).
    pub async fn add(&self, q: Quest) {
        self.quests.write().await.insert(q.id, q);
    }
}

impl Default for QuestStore {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl QuestManager for QuestStore {
    async fn available_quests(&self) -> Result<Vec<Quest>> {
        Ok(self.quests.read().await.values().filter(|q| q.is_available()).cloned().collect())
    }

    async fn accept_quest(&self, qid: Uuid, agent_id: Uuid) -> Result<()> {
        let mut qs = self.quests.write().await;
        let q = qs.get_mut(&qid).ok_or_else(|| FfError::QuestNotFound(qid.to_string()))?;
        if !q.is_available() { return Err(FfError::QuestUnavailable(qid.to_string())); }
        q.status      = QuestStatus::Active { progress_pct: 0 };
        q.accepted_by = Some(agent_id);
        Ok(())
    }

    async fn update_progress(&self, qid: Uuid, pct: u8) -> Result<()> {
        let mut qs = self.quests.write().await;
        let q = qs.get_mut(&qid).ok_or_else(|| FfError::QuestNotFound(qid.to_string()))?;
        q.status = QuestStatus::Active { progress_pct: pct };
        Ok(())
    }

    async fn complete_quest(&self, qid: Uuid, tick: u64) -> Result<()> {
        let mut qs = self.quests.write().await;
        let q = qs.get_mut(&qid).ok_or_else(|| FfError::QuestNotFound(qid.to_string()))?;
        q.status = QuestStatus::Completed { at_tick: tick };
        Ok(())
    }

    async fn fail_quest(&self, qid: Uuid, reason: &str) -> Result<()> {
        let mut qs = self.quests.write().await;
        let q = qs.get_mut(&qid).ok_or_else(|| FfError::QuestNotFound(qid.to_string()))?;
        q.status = QuestStatus::Failed { reason: reason.to_string() };
        Ok(())
    }

    async fn generate_quest(&self, world: &WorldState, _agent: &Agent) -> Result<Quest> {
        let (title, desc, obj) = match world.epoch.number % 4 {
            0 => ("Eisenabbau",      "Baue 10 Eisenerz ab.",         QuestObjective::MineBlocks   { material: "Iron".into(),         count: 10 }),
            1 => ("Gold verdienen",  "Verdiene 50 Gold durch Handel.", QuestObjective::Earn         { amount: 50 }),
            2 => ("Expedition",      "Erkunde die Vulkanöde.",        QuestObjective::Explore      { target: "VolcanicWastes".into(), radius: 50 }),
            _ => ("Code-Herausforderung", "Berechne Fibonacci(10).", QuestObjective::Freeform     { description: "Fibonacci(10)".into() }),
        };
        let q = Quest::new(title, desc, obj, QuestReward { gold: 40, xp: 80, items: vec![] });
        self.quests.write().await.insert(q.id, q.clone());
        Ok(q)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::world::WorldState;

    #[tokio::test]
    async fn generate_and_accept() {
        let store = QuestStore::new();
        let world = WorldState::new(42);
        let agent_id = Uuid::new_v4();
        let agent = types::agent::Agent::new("Tester", types::agent::AgentKind::Claude);
        let q = store.generate_quest(&world, &agent).await.unwrap();
        assert!(q.is_available());
        store.accept_quest(q.id, agent_id).await.unwrap();
        let qs = store.available_quests().await.unwrap();
        assert!(qs.is_empty(), "Quest sollte nicht mehr verfügbar sein");
    }
}
