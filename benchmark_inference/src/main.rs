#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use sn_core::types::conversation::Conversation;
use sn_core::types::message::{Message, MessageBuilder, MessageRole};
use sn_inference::runner::Runner;

fn similarity(runner: &Runner) -> Result<(), Box<dyn std::error::Error>> {
    let queries = vec![
        String::from("What is the capital of China?"),
        String::from("Explain gravity"),
    ];
    let documents = vec![
        String::from("The capital of China is Beijing."),
        String::from(
            "Gravity is a force that attracts two bodies towards each other. It gives weight to physical objects and is responsible for the movement of planets around the sun.",
        ),
    ];

    let result = runner.generate_similarity(&queries, &documents)?;
    println!("Similarity Results: {:?}", result);
    Ok(())
}
fn embedding(runner: &Runner) -> Result<(), Box<dyn std::error::Error>> {
    let sentences = vec![
        "The cat sits on the mat.".to_string(),
        "Rust makes systems programming safe.".to_string(),
        "Machine learning is fun!".to_string(),
        "I love open source software.".to_string(),
        "Tomorrow will be a sunny day.".to_string(),
    ];
    let embedding = runner.generate_embeddings(&sentences)?;
    println!("Embedding Results: {:?}", embedding);
    Ok(())
}
fn chat(runner: &Runner) -> Result<(), Box<dyn std::error::Error>> {
    let conversation = Conversation::from_message(
        MessageBuilder::default()
            .content("Hello, who are you?".to_string())
            .role(MessageRole::User)
            .build()
            .unwrap(),
    );
    //let model_id = runner.load_model_name("models--Qwen--Qwen3-1.7B-MLX-4bit", None)?;
    let model_id = runner.load_model_name("models-llama-3.1-8B-Instruct-4bit", None)?;
    let text = runner.generate_text(&model_id, &conversation, None, None)?;
    println!("Chat Response: {}", text.0);
    Ok(())
}

fn main() {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();
    let runner = Runner::new().unwrap();

    match (|| {
        // embedding(&runner)?;
        similarity(&runner)?;
        //chat(&runner)?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })() {
        Err(e) => println!("{e}"),
        Ok(_) => (),
    }
}
