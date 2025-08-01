use crate::error::Error;
use minijinja::Environment;
use serde_json::{json, Value};
use sn_core::conversation::conversation::Conversation;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

pub struct Document {
    pub title: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub function: ToolFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub arguments: HashMap<String, String>,
}


#[derive(Debug, Clone)]
pub enum Tool {
    Schema(Value),
    Function(fn() -> Value), // Placeholder for runtime callables
}

pub fn render_chat_template(
    conversations: &Conversation,
    tools: Option<&[Tool]>,
    documents: Option<&[Document]>,
    env: &Environment,
    template_str: &str,
    return_assistant_tokens_mask: bool,
    continue_final_message: bool,
    // other args like add_generation_prompt, kwargs could be added here
) -> Result<(Vec<String>, Option<Vec<(usize, usize)>>), Error> {
    // Compile template once
    let template = env.get_template(template_str)?;

    let mut rendered_vec = Vec::with_capacity(conversations.messages.len());
    let mut assistant_indices_vec = if return_assistant_tokens_mask {
        Some(Vec::with_capacity(conversations.messages.len()))
    } else {
        None
    };

    for messages in &conversations.messages {
        // Context for rendering: build a HashMap
        let mut context = HashMap::new();
        context.insert("messages", json!(vec![messages]) );

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

        let rendered = template.render(context)?;
        rendered_vec.push(rendered);
    }

    Ok((rendered_vec, assistant_indices_vec))
}
