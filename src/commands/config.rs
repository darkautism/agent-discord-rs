use super::SlashCommand;
use async_trait::async_trait;
use serenity::all::{
    CommandInteraction, Context, CreateActionRow, CreateSelectMenu, CreateSelectMenuKind,
    CreateSelectMenuOption, EditInteractionResponse,
};

use crate::agent::AgentType;

pub struct ConfigCommand;

#[async_trait]
impl SlashCommand for ConfigCommand {
    fn name(&self) -> &'static str {
        "config"
    }

    fn description(&self, i18n: &crate::i18n::I18n) -> String {
        i18n.get("cmd_config_desc")
    }

    async fn execute(
        &self,
        ctx: &Context,
        command: &CommandInteraction,
        state: &crate::AppState,
    ) -> anyhow::Result<()> {
        command.defer_ephemeral(&ctx.http).await?;

        let channel_id_str = command.channel_id.to_string();
        let channel_config = crate::commands::agent::ChannelConfig::load()
            .await
            .unwrap_or_default();
        let backend = channel_config.get_agent_type(&channel_id_str);
        let assistant_name = channel_config
            .channels
            .get(&channel_id_str)
            .and_then(|e| e.assistant_name.clone())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| state.config.assistant_name.clone());
        let mention_only = state
            .auth
            .get_channel_mention_only(&channel_id_str)
            .unwrap_or(true);

        let i18n = state.i18n.read().await;
        let status = i18n.get_args(
            "config_current",
            &[
                backend.to_string(),
                if mention_only {
                    i18n.get("config_mention_on")
                } else {
                    i18n.get("config_mention_off")
                },
                assistant_name,
            ],
        );

        let backend_menu = CreateSelectMenu::new(
            "config_backend_select",
            CreateSelectMenuKind::String {
                options: vec![
                    CreateSelectMenuOption::new(i18n.get("agent_choice_kilo"), "kilo"),
                    CreateSelectMenuOption::new(i18n.get("agent_choice_copilot"), "copilot"),
                    CreateSelectMenuOption::new(i18n.get("agent_choice_pi"), "pi"),
                    CreateSelectMenuOption::new(i18n.get("agent_choice_opencode"), "opencode"),
                ],
            },
        )
        .placeholder(i18n.get("config_backend_placeholder"))
        .min_values(1)
        .max_values(1);

        let mention_menu = CreateSelectMenu::new(
            "config_mention_select",
            CreateSelectMenuKind::String {
                options: vec![
                    CreateSelectMenuOption::new(i18n.get("config_mention_on"), "on"),
                    CreateSelectMenuOption::new(i18n.get("config_mention_off"), "off"),
                ],
            },
        )
        .placeholder(i18n.get("config_mention_placeholder"))
        .min_values(1)
        .max_values(1);

        let assistant_menu = CreateSelectMenu::new(
            "config_assistant_select",
            CreateSelectMenuKind::String {
                options: vec![
                    CreateSelectMenuOption::new(i18n.get("config_assistant_default"), "default"),
                    CreateSelectMenuOption::new("Agent", "Agent"),
                    CreateSelectMenuOption::new("Assistant", "Assistant"),
                    CreateSelectMenuOption::new("Coder", "Coder"),
                    CreateSelectMenuOption::new("Analyst", "Analyst"),
                ],
            },
        )
        .placeholder(i18n.get("config_assistant_placeholder"))
        .min_values(1)
        .max_values(1);

        command
            .edit_response(
                &ctx.http,
                EditInteractionResponse::new()
                    .content(status)
                    .components(vec![
                        CreateActionRow::SelectMenu(backend_menu),
                        CreateActionRow::SelectMenu(mention_menu),
                        CreateActionRow::SelectMenu(assistant_menu),
                    ]),
            )
            .await?;

        Ok(())
    }
}

pub async fn handle_config_select(
    ctx: &Context,
    interaction: &serenity::all::ComponentInteraction,
    state: &crate::AppState,
) -> anyhow::Result<()> {
    interaction.defer_ephemeral(&ctx.http).await?;

    let custom_id = interaction.data.custom_id.as_str();
    let value = match &interaction.data.kind {
        serenity::all::ComponentInteractionDataKind::StringSelect { values } => {
            values.first().cloned()
        }
        _ => None,
    };
    let Some(value) = value else {
        return Ok(());
    };

    let channel_id_u64 = interaction.channel_id.get();
    let channel_id_str = interaction.channel_id.to_string();

    match custom_id {
        "config_backend_select" => {
            let selected: AgentType = value.parse()?;
            let mut channel_config = crate::commands::agent::ChannelConfig::load()
                .await
                .unwrap_or_default();
            let current = channel_config.get_agent_type(&channel_id_str);

            let msg = if current == selected {
                let i18n = state.i18n.read().await;
                i18n.get_args("agent_already", &[selected.to_string()])
            } else {
                channel_config.set_agent_type(&channel_id_str, selected.clone());
                state.session_manager.remove_session(channel_id_u64).await;

                match state
                    .session_manager
                    .get_or_create_session(channel_id_u64, selected.clone(), &state.backend_manager)
                    .await
                {
                    Ok(_) => {
                        channel_config.save().await?;
                        let i18n = state.i18n.read().await;
                        i18n.get_args("config_backend_set", &[selected.to_string()])
                    }
                    Err(e) => {
                        let i18n = state.i18n.read().await;
                        crate::commands::agent::build_backend_error_message(
                            &i18n,
                            selected,
                            &e.to_string(),
                            state.config.opencode.port,
                        )
                    }
                }
            };

            interaction
                .edit_response(&ctx.http, EditInteractionResponse::new().content(msg))
                .await?;
        }
        "config_mention_select" => {
            let enable = value == "on";
            let msg = {
                let i18n = state.i18n.read().await;
                match state.auth.set_mention_only(&channel_id_str, enable) {
                    Ok(_) => i18n.get(if enable { "mention_on" } else { "mention_off" }),
                    Err(_) => i18n.get("mention_not_auth"),
                }
            };

            interaction
                .edit_response(&ctx.http, EditInteractionResponse::new().content(msg))
                .await?;
        }
        "config_assistant_select" => {
            let mut channel_config = crate::commands::agent::ChannelConfig::load()
                .await
                .unwrap_or_default();
            channel_config.set_agent_type(
                &channel_id_str,
                channel_config.get_agent_type(&channel_id_str),
            );
            if let Some(entry) = channel_config.channels.get_mut(&channel_id_str) {
                entry.assistant_name = if value == "default" {
                    None
                } else {
                    Some(value.clone())
                };
            }
            channel_config.save().await?;

            let msg = {
                let i18n = state.i18n.read().await;
                let chosen = if value == "default" {
                    state.config.assistant_name.clone()
                } else {
                    value
                };
                i18n.get_args("config_assistant_set", &[chosen])
            };

            interaction
                .edit_response(&ctx.http, EditInteractionResponse::new().content(msg))
                .await?;
        }
        _ => {}
    }

    Ok(())
}
