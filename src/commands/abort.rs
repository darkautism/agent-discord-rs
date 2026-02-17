use super::SlashCommand;
use async_trait::async_trait;
use serenity::all::{CommandInteraction, Context, EditInteractionResponse};
use std::sync::Arc;

use crate::agent::AiAgent;

pub struct AbortCommand;

#[async_trait]
impl SlashCommand for AbortCommand {
    fn name(&self) -> &'static str {
        "abort"
    }

    fn description(&self, i18n: &crate::i18n::I18n) -> String {
        i18n.get("cmd_abort_desc")
    }

    async fn execute(
        &self,
        ctx: &Context,
        command: &CommandInteraction,
        agent: Arc<dyn AiAgent>,
        state: &crate::AppState,
    ) -> anyhow::Result<()> {
        command.defer_ephemeral(&ctx.http).await?;

        agent.abort().await?;

        let i18n = state.i18n.read().await;
        let msg = i18n.get("abort_success");
        drop(i18n);

        command
            .edit_response(&ctx.http, EditInteractionResponse::new().content(msg))
            .await?;

        Ok(())
    }
}
