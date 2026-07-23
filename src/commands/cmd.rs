use crate::state::AppState;
use async_trait::async_trait;
use linkme::distributed_slice;
use whatsapp_rust::SendResult;
use std::sync::LazyLock;
use std::{collections::HashMap, sync::Arc};
use wacore::types::message::MessageInfo;
use waproto::whatsapp as wa;
use waproto::whatsapp::Message;
use waproto::whatsapp::message::ReactionMessage;
use whatsapp_rust::client::Client;

#[derive(Clone)]
pub struct Context<'a> {
    pub client: Arc<Client>,
    pub msg: &'a Message,
    pub info: &'a MessageInfo,
    pub state: Arc<AppState>,
    pub body: &'a str,
    pub args: &'a Vec<&'a str>,
}

impl<'a> Context<'a> {
    pub async fn react(&self, emoji: &str) -> anyhow::Result<SendResult> {
        let reaction = wa::Message {
            reaction_message: Some(ReactionMessage {
                key: Some(wa::MessageKey {
                    remote_jid: Some(self.info.source.chat.to_string()),
                    from_me: Some(false),
                    id: Some(self.info.id.to_string()),
                    participant: Some(self.info.source.sender.to_string()),
                }),
                text: Some(emoji.to_string()),
                sender_timestamp_ms: Some(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as i64,
                ),
                ..Default::default()
            }),
            ..Default::default()
        };

        self.client
            .send_message(self.info.source.chat.clone(), reaction)
            .await
    }
    pub async fn reply(&self, text: &str) -> anyhow::Result<String> {
        let msg_id = crate::send_msg!(
            context: self,
            dst: self.info.source.chat,
            text: text,
            reply: true
        )
        .await?;
        Ok(msg_id.message_id)
    }
}
#[distributed_slice]
pub static COMMANDS: [&(dyn Command + Sync)] = [..];

#[distributed_slice]
pub static INTERCEPTORS: [&(dyn Command + Sync)] = [..];

#[async_trait]
pub trait Command: Send + Sync {
    fn name(&self) -> &str;
    fn aliases(&self) -> &[&str];
    fn category(&self) -> &str;
    async fn intercept(&self, _ctx: Context<'_>) -> anyhow::Result<bool> {
        Ok(false)
    }
    async fn execute(&self, ctx: Context<'_>) -> anyhow::Result<()>;
}

#[macro_export]
macro_rules! cmd {
    ($struct_name:ident, name: $name:expr, aliases: [$($alias:expr),*], category: $cat:expr,
     intercept: |$ctx_int:ident| $int_body:block,
     execute: |$ctx:ident| $exec_body:block) => {
        pub struct $struct_name;

        #[async_trait::async_trait]
        impl $crate::commands::cmd::Command for $struct_name {
            fn name(&self) -> &str { $name }
            fn aliases(&self) -> &[&str] { &[$($alias),*] }
            fn category(&self) -> &str { $cat }

            async fn intercept(&self, $ctx_int: $crate::commands::cmd::Context<'_>) -> anyhow::Result<bool> {
                $int_body
            }

            async fn execute(&self, $ctx: $crate::commands::cmd::Context<'_>) -> anyhow::Result<()> {
                $exec_body;
                Ok(())
            }
        }

        #[linkme::distributed_slice($crate::commands::cmd::COMMANDS)]
        static COMMAND: &(dyn $crate::commands::cmd::Command + Sync) = &$struct_name;

        #[linkme::distributed_slice($crate::commands::cmd::INTERCEPTORS)]
        static INTERCEPTOR: &(dyn $crate::commands::cmd::Command + Sync) = &$struct_name;
    };

    ($struct_name:ident, name: $name:expr, aliases: [$($alias:expr),*], category: $cat:expr, execute: |$ctx:ident| $body:block) => {
        pub struct $struct_name;

        #[async_trait::async_trait]
        impl $crate::commands::cmd::Command for $struct_name {
            fn name(&self) -> &str { $name }
            fn aliases(&self) -> &[&str] { &[$($alias),*] }
            fn category(&self) -> &str { $cat }
            async fn execute(&self, $ctx: $crate::commands::cmd::Context<'_> ) -> anyhow::Result<()> {
                $body;
                Ok(())
            }
        }

        #[linkme::distributed_slice($crate::commands::cmd::COMMANDS)]
        static COMMAND: &(dyn $crate::commands::cmd::Command + Sync) = &$struct_name;
    };
}

pub static COMMAND_MAP: LazyLock<HashMap<String, &'static (dyn Command + Sync)>> =
    LazyLock::new(|| {
        let mut map = HashMap::with_capacity(COMMANDS.len() * 2);
        for &cmd in COMMANDS {
            map.insert(cmd.name().to_lowercase(), cmd);
            for alias in cmd.aliases() {
                map.insert(alias.to_lowercase(), cmd);
            }
        }
        map
    });
