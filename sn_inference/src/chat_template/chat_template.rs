use crate::config::config::Config;
use crate::error::Result;
use minijinja::Environment;
use serde_json::{Value, json};
use sn_core::types::conversation::Conversation;
use sn_core::types::document::Document;
use sn_core::types::tool::Tool;
use std::collections::HashMap;

fn convert_python_to_rust_template(template: &str) -> String {
    template
        // remove think from template & messages (used for Qwen3)
        .replace(
            "message.content.split('</think>')[-1].lstrip('\\n')",
            "(message.content | split('</think>'))[-1] | trim",
        )
        .replace(
            "message.content.startswith('<tool_response>')",
            "message.content[:15] == '<tool_response>'",
        )
        .replace(
            "message.content.endswith('</tool_response>')",
            "message.content[-16:] == '</tool_response>'",
        )
}

#[derive(Debug, Clone)]
pub struct ChatTemplate {
    name: String,
    template: String,
}

impl ChatTemplate {
    pub fn new(config: &Config) -> Result<Self> {
        //chat_template: &str
        let chat_template = config.tokenizer_custom.get_chat_template();
        let template = convert_python_to_rust_template(chat_template);

        Ok(ChatTemplate {
            template,
            name: "chat".to_string(),
        })
    }

    fn render_chat_template(
        &self,
        conversations: &Conversation,
        tools: Option<&[Tool]>,
        documents: Option<&[Document]>,
        add_generation_prompt: Option<bool>,
        // other args like add_generation_prompt, kwargs could be added here
    ) -> Result<String> {
        let mut env = Environment::new();
        env.add_template(self.name.as_str(), self.template.as_str())?;

        // Compile template once
        let template = env.get_template(self.name.as_str())?;

        // Context for rendering: build a HashMap
        let mut context = HashMap::new();
        let messages = &conversations.messages;
        context.insert("messages", json!(messages));

        // Tools
        if let Some(tools) = tools {
            let tools_json: Vec<Value> = tools
                .iter()
                .filter_map(|tool| match tool {
                    Tool::Schema(v) => Some(v.clone()),
                    Tool::Function(_) => None,
                })
                .collect();
            context.insert("custom_tools", json!(tools_json));
        }

        // Documents
        if let Some(docs) = documents {
            let docs_json: Vec<Value> = docs
                .iter()
                .map(|d| json!({ "title": d.title, "text": d.text }))
                .collect();
            context.insert("documents", json!(docs_json));
        }

        if let Some(add_generation_prompt) = add_generation_prompt {
            context.insert(
                "add_generation_prompt",
                json!({"add_generation_prompt": add_generation_prompt}),
            );
        }

        Ok(template.render(context)?)
    }

    pub fn apply_chat_template(
        &self,
        conversations: &Conversation,
        tools: Option<&[Tool]>,
        documents: Option<&[Document]>,
    ) -> Result<String> {
        self.render_chat_template(&conversations, tools, documents, Some(true))
    }
}
