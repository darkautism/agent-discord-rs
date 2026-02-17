use super::SlashCommand;
use async_trait::async_trait;
use serenity::all::{CommandInteraction, Context, EditInteractionResponse};
use std::sync::Arc;

use super::agent::ChannelConfig;
use crate::agent::AiAgent;
use crate::migrate;

pub struct ClearCommand;

#[async_trait]
impl SlashCommand for ClearCommand {
    fn name(&self) -> &'static str {
        "clear"
    }

    fn description(&self, i18n: &crate::i18n::I18n) -> String {
        i18n.get("cmd_clear_desc")
    }

    async fn execute(
        &self,
        ctx: &Context,
        command: &CommandInteraction,
        agent: Arc<dyn AiAgent>,
        state: &crate::AppState,
    ) -> anyhow::Result<()> {
        command.defer_ephemeral(&ctx.http).await?;

        let channel_id = command.channel_id.get();

        // 1. 清除後端 session
        agent.clear().await?;

        // 2. 移除記憶體快取
        state.session_manager.remove_session(channel_id).await;

        // 3. 刪除本地 session 檔案
        let agent_type = agent.agent_type();
        let session_file =
            migrate::get_sessions_dir(agent_type).join(format!("discord-rs-{}.jsonl", channel_id));

        if session_file.exists() {
            tokio::fs::remove_file(&session_file).await.ok();
        }

        // 4. 清除持久化配置中的 ID
        if let Ok(mut config) = ChannelConfig::load().await {
            if let Some(entry) = config.channels.get_mut(&channel_id.to_string()) {
                entry.session_id = None;
                let _ = config.save().await;
            }
        }

        let i18n = state.i18n.read().await;
        let msg = i18n.get("clear_success");
        drop(i18n);

        command
            .edit_response(&ctx.http, EditInteractionResponse::new().content(msg))
            .await?;

        Ok(())
    }
}
