use sqlx::SqlitePool;
use anyhow::Result;
use crate::models::StateMachine;

pub async fn insert_machine(pool: &SqlitePool, sm: &StateMachine) -> Result<()> {
    let source  = serde_json::to_string(&sm.source)?;
    let states  = serde_json::to_string(&sm.states)?;
    let trans   = serde_json::to_string(&sm.transitions)?;
    let tags    = serde_json::to_string(&sm.tags)?;

    sqlx::query!(
        "INSERT OR REPLACE INTO state_machines
         (id, name, source_json, states_json, transitions_json, initial_state, context_type, tags_json, ai_summary, created_at, updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
        sm.id, sm.name, source, states, trans, sm.initial_state,
        sm.context_type, tags, sm.ai_summary, sm.created_at, sm.updated_at
    )
    .execute(pool).await?;
    Ok(())
}

pub async fn list_machines(pool: &SqlitePool) -> Result<Vec<StateMachine>> {
    let rows = sqlx::query!(
        "SELECT id, name, source_json, states_json, transitions_json, initial_state,
                context_type, tags_json, ai_summary, created_at, updated_at
         FROM state_machines ORDER BY updated_at DESC"
    )
    .fetch_all(pool).await?;

    rows.into_iter().map(|r| {
        Ok(StateMachine {
            id: r.id,
            name: r.name,
            source: serde_json::from_str(&r.source_json)?,
            states: serde_json::from_str(&r.states_json)?,
            transitions: serde_json::from_str(&r.transitions_json)?,
            initial_state: r.initial_state,
            context_type: r.context_type,
            tags: serde_json::from_str(&r.tags_json).unwrap_or_default(),
            ai_summary: r.ai_summary,
            created_at: r.created_at.parse()?,
            updated_at: r.updated_at.parse()?,
        })
    }).collect()
}

pub async fn get_machine(pool: &SqlitePool, id: &str) -> Result<Option<StateMachine>> {
    let row = sqlx::query!(
        "SELECT id, name, source_json, states_json, transitions_json, initial_state,
                context_type, tags_json, ai_summary, created_at, updated_at
         FROM state_machines WHERE id = ?1",
        id
    )
    .fetch_optional(pool).await?;

    let Some(r) = row else { return Ok(None) };
    Ok(Some(StateMachine {
        id: r.id,
        name: r.name,
        source: serde_json::from_str(&r.source_json)?,
        states: serde_json::from_str(&r.states_json)?,
        transitions: serde_json::from_str(&r.transitions_json)?,
        initial_state: r.initial_state,
        context_type: r.context_type,
        tags: serde_json::from_str(&r.tags_json).unwrap_or_default(),
        ai_summary: r.ai_summary,
        created_at: r.created_at.parse()?,
        updated_at: r.updated_at.parse()?,
    }))
}

pub async fn delete_machine(pool: &SqlitePool, id: &str) -> Result<()> {
    sqlx::query!("DELETE FROM state_machines WHERE id = ?1", id)
        .execute(pool).await?;
    Ok(())
}

pub async fn get_setting(pool: &SqlitePool, key: &str) -> Result<Option<String>> {
    let row = sqlx::query!("SELECT value FROM app_settings WHERE key = ?1", key)
        .fetch_optional(pool).await?;
    Ok(row.map(|r| r.value))
}

pub async fn set_setting(pool: &SqlitePool, key: &str, value: &str) -> Result<()> {
    sqlx::query!("INSERT OR REPLACE INTO app_settings (key, value) VALUES (?1, ?2)", key, value)
        .execute(pool).await?;
    Ok(())
}
