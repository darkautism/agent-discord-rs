use super::SlashCommand;
use async_trait::async_trait;
use serenity::all::{CommandInteraction, Context, EditInteractionResponse};

use super::agent::ChannelConfig;
use crate::migrate;

pub struct NewCommand;

#[async_trait]
impl SlashCommand for NewCommand {
    fn name(&self) -> &'static str {
        "new"
    }

    fn description(&self, i18n: &crate::i18n::I18n) -> String {
        i18n.get("cmd_new_desc")
    }

    async fn execute(
        &self,
        ctx: &Context,
        command: &CommandInteraction,
        state: &crate::AppState,
    ) -> anyhow::Result<()> {
        command.defer_ephemeral(&ctx.http).await?;

        let channel_id_u64 = command.channel_id.get();
        let channel_id_str = channel_id_u64.to_string();
        let channel_config = crate::commands::agent::ChannelConfig::load()
            .await
            .unwrap_or_default();
        let agent_type = channel_config.get_agent_type(&channel_id_str);

        // 1. 清除後端 session（讓後端有機會釋放資源）
        {
            let (agent, _) = state
                .session_manager
                .get_or_create_session(channel_id_u64, agent_type.clone(), &state.backend_manager)
                .await?;
            let _ = agent.clear().await;
        }

        // 2. 移除記憶體快取（觸發 Drop，中止 subprocess）
        state.session_manager.remove_session(channel_id_u64).await;

        // 3. 對 Pi backend：重新命名舊 session 檔案，讓新 agent 建立全新檔案
        if agent_type.to_string() == "pi" {
            let session_file = migrate::get_sessions_dir("pi")
                .join(format!("discord-rs-{}.jsonl", channel_id_u64));
            if session_file.exists() {
                let ts = chrono::Utc::now().format("%Y%m%d%H%M%S");
                let backup = session_file.with_extension(format!("jsonl.{ts}.bak"));
                if let Err(e) = tokio::fs::rename(&session_file, &backup).await {
                    let i18n = state.i18n.read().await;
                    let err_msg = format!("{}: {}", i18n.get("new_rename_failed"), e);
                    drop(i18n);
                    command
                        .edit_response(&ctx.http, EditInteractionResponse::new().content(err_msg))
                        .await?;
                    return Ok(());
                }
            }
        }

        // 4. 清除持久化配置中的 session_id（強制後端建立新 session）
        if let Ok(mut config) = ChannelConfig::load().await {
            if let Some(entry) = config.channels.get_mut(&channel_id_str) {
                entry.session_id = None;
                let _ = config.save().await;
            }
        }

        // 5. 建立新 session（會觸發 Pi 建立全新 JSONL 或 OpenCode/Kilo 建立新 HTTP session）
        match state
            .session_manager
            .get_or_create_session(channel_id_u64, agent_type, &state.backend_manager)
            .await
        {
            Ok(_) => {
                let i18n = state.i18n.read().await;
                let msg = i18n.get("new_success");
                drop(i18n);
                command
                    .edit_response(&ctx.http, EditInteractionResponse::new().content(msg))
                    .await?;
            }
            Err(e) => {
                let i18n = state.i18n.read().await;
                let msg = format!("{}: {}", i18n.get("new_failed"), e);
                drop(i18n);
                command
                    .edit_response(&ctx.http, EditInteractionResponse::new().content(msg))
                    .await?;
            }
        }

        Ok(())
    }
}
