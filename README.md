# Agent Discord (Rust)

A high-performance Discord Bot daemon developed in Rust, designed to bridge and manage multiple AI Agent backends.

## Core Features

- **Multi-backend Integration**: Unified interface for managing Pi (CLI), OpenCode (API), and Kilo (API) backends.
- **Real-time State Rendering**: Synchronized display of AI reasoning streams and tool execution (Tool Use) status.
- **Session Lifecycle Management**: Dynamic backend switching, model selection, thinking level configuration, and command-based context compression (`/compact`).
- **I18n Support**: Seamless switching between Traditional Chinese (zh-TW) and English (en), with automatic re-registration of Slash Commands to update localized descriptions.

## Commands (Slash Commands)

- `/agent`: Switch the active AI Agent backend.
- `/model`: Switch the AI model used in the current channel.
- `/thinking`: Set the AI thinking depth (subject to model capability).
- `/compact`: Manually trigger context compression to save tokens.
- `/language`: Switch the bot interface language.
- `/clear`: Completely wipe current session state and local JSONL history.
- `/mention_only`: Toggle whether to respond only when mentioned (@).

## Deployment

### Prerequisites

1.  **Rust Toolchain**: Install via [rust-lang.org](https://www.rust-lang.org/tools/install).
2.  **Discord Bot Token**:
    -   Create an application at the [Discord Developer Portal](https://discord.com/developers/applications).
    -   Under **Bot**, enable the following **Privileged Gateway Intents**:
        -   `Presence Intent` (Optional)
        -   `Server Members Intent`
        -   `Message Content Intent` (Required)
3.  **AI Backends**: Install at least one of the following:
    -   **Pi**: [github.com/mariozechner/pi-coding-agent](https://github.com/mariozechner/pi-coding-agent)
    -   **OpenCode**: `npm install -g @opencode-ai/cli`
    -   **Kilo**: `npm install -g @kilocode/cli`

### Installation

Install via [crates.io](https://crates.io/crates/agent-discord-rs):
```bash
cargo install agent-discord-rs
```

Or build from source:
```bash
git clone https://github.com/darkautism/pi-discord-rs.git
cd pi-discord-rs
cargo install --path .
```

### Initial Setup

1.  **Generate Config**: Run the bot for the first time to create the default config directory:
    ```bash
    agent-discord run
    ```
2.  **Edit Config**: Locate `config.toml` (typically in `~/.config/agent-discord-rs/config.toml`) and paste your `discord_token`.
3.  **Authentication**:
    -   Invite the bot to your server.
    -   **Mention (@) the bot in a channel** to trigger the authentication prompt.
    -   Follow the instructions and run:
      ```bash
      agent-discord auth <TOKEN_FROM_DISCORD>
      ```

### Running the Bot

```bash
# Start the bot
agent-discord run

# Background daemon (Linux/Systemd)
agent-discord daemon enable
```

## Acknowledgments

This project relies on the following backends for its AI capabilities. Special thanks to the developers:

- **[Pi](https://github.com/mariozechner/pi-coding-agent)**: The core for local automation and RPC calls.
- **OpenCode**: A robust HTTP/SSE backend with full tool-calling support.
- **Kilo**: A specialized backend implementation optimized for long-running sessions.

## License

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for details.
