#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use sn_core::types::conversation::Conversation;
use sn_core::types::message;
use sn_core::types::message::{Message, MessageRole};
use sn_inference::runner::Runner;
fn main() {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();
    let mut runner = Runner::new();

    let conversation = Conversation::from_message(Message {
        //content: String::from("Hi, my name is <name>."),
        content: String::from("i have a dream, that one day this..."),
        role: MessageRole::User,
        stats: None,
    });

    match (|| {
        let model_id = runner.load_model_name("models-llama-3.1-8B-Instruct-4bit", None)?;
        runner.generate_text(&model_id, &conversation, None)?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })() {
        Err(e) => println!("{e}"),
        Ok(_) => (),
    }
}
