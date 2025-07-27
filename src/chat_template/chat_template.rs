use crate::conversation::Conversation;
use crate::error::Error;
use minijinja::Environment;
use serde_json::Value;
use std::collections::HashMap;

pub struct Document {
    pub title: String,
    pub text: String,
}

pub enum Tool {
    Schema(Value),
    Function(fn() -> Value), // Simplified placeholder
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

    // Process tools into a JSON-like Vec or skip
    let tool_schemas = if let Some(tools) = tools {
        let mut schemas = Vec::new();
        for tool in tools {
            match tool {
                Tool::Schema(schema) => schemas.push(schema.clone()),
                Tool::Function(f) => {
                    // Call function and convert result to Value schema
                    let schema = f();
                    schemas.push(schema);
                }
            }
        }
        Some(schemas)
    } else {
        None
    };

    // Validate documents - ensure they have title and text (already struct-typed, so likely safe)

    let mut rendered_vec = Vec::with_capacity(conversations.len());
    let mut assistant_indices_vec = if return_assistant_tokens_mask {
        Some(Vec::with_capacity(conversations.len()))
    } else {
        None
    };

    for chat in conversations.as_array() {
        // Context for rendering: build a HashMap
        let mut context = HashMap::new();
        context.insert("messages", conversations.as_array());

        if let Some(schemas) = &tool_schemas {
            //context.insert("tools", serde_json::to_value(schemas)?);
        }

        //if let Some(docs) = &documents {
        //    context.insert("documents", serde_json::to_value(docs)?);
        //}

        // Render template
        let rendered = template.render(context)?;

        if return_assistant_tokens_mask {
            // Use marker strings to track assistant tokens (simulate {%- generation -%} block)
            let start_tag = "__GEN_START__";
            let end_tag = "__GEN_END__";

            // Find indices of assistant tokens inside the rendered output
            if let (Some(start), Some(end)) = (rendered.find(start_tag), rendered.find(end_tag)) {
                // Compute indices relative to clean string (markers removed)
                let content_start = start;
                let content_end = end - start_tag.len();

                // Remove markers for clean output
                let clean = rendered.replace(start_tag, "").replace(end_tag, "");
                // Optionally trim after final message
                let final_output = if continue_final_message {
                    if let Some(last_msg) = conversations.last_message() {
                        if let final_message = last_msg.content {
                            if let Some(pos) = clean.rfind(final_message.trim()) {
                                // Adjust length to preserve spacing if necessary
                                let slice_end = pos + final_message.trim().len();
                                clean[..slice_end].to_string()
                            } else {
                                clean
                            }
                        } else {
                            clean
                        }
                    } else {
                        clean
                    }
                } else {
                    clean
                };

                rendered_vec.push(final_output);
                if let Some(ref mut assistant_vec) = assistant_indices_vec {
                    assistant_vec.push((content_start, content_end));
                }
            } else {
                // Markers missing, fallback to just rendered
                rendered_vec.push(rendered);
                if let Some(ref mut assistant_vec) = assistant_indices_vec {
                    assistant_vec.push((0, 0));
                }
            }
        } else {
            // No token tracking requested
            // Optionally handle continue_final_message here too if needed
            let final_output = if continue_final_message {
                if let Some(last_msg) = &conversations.last_message() {
                    if let final_message = &last_msg.content {
                        if let Some(pos) = rendered.rfind(final_message.trim()) {
                            let slice_end = pos + final_message.trim().len();
                            rendered[..slice_end].to_string()
                        } else {
                            rendered.clone()
                        }
                    } else {
                        rendered.clone()
                    }
                } else {
                    rendered.clone()
                }
            } else {
                rendered.clone()
            };

            rendered_vec.push(final_output);
        }
    }

    Ok((rendered_vec, assistant_indices_vec))
}
