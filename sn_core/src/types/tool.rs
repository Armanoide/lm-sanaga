/*
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub function: ToolFunction,
}

#[derive(Debug, Clone)]
pub struct ToolFunction {
    pub name: String,
    pub arguments: HashMap<String, String>,
}*/
use serde_json::Value;

#[derive(Debug, Clone)]
pub enum Tool {
    Schema(Value),
    Function(fn() -> Value), // Placeholder for runtime callables
}
