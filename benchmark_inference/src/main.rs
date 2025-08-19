#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use sn_core::types::conversation::Conversation;
use sn_core::types::message::{Message, MessageRole};
use sn_inference::runner::Runner;

fn embedding(runner: &Runner) -> Result<(), Box<dyn std::error::Error>> {
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

    //let model_id = runner.load_model_name("models--mlx-community--Qwen3-Embedding-0.6B-4bit-DWQ", None)?;
    let model_id = runner.load_model_name("models--Qwen--Qwen3-Embedding-0.6B", None)?;
    let embeddings = runner.generate_similarity(&model_id, &queries, &documents)?;
    //println!("Query Embeddings: {:?}", embeddings);
    /*let result = runner.similarity(
        &model_id,
        &query_embeddings,
        &document_embeddings,
    )?;*/
    //println!("Similarity Results: {:?}", result);
    Ok(())
}
fn chat(runner: &Runner) -> Result<(), Box<dyn std::error::Error>> {
    let conversation = Conversation::from_message(Message {
        //content: String::from("Hi, my name is <name>."),
        content: String::from("What is the capital of China?"),
        role: MessageRole::User,
        stats: None,
    });
    //let model_id = runner.load_model_name("models--Qwen--Qwen3-1.7B-MLX-4bit", None)?;
    let model_id = runner.load_model_name("models-llama-3.1-8B-Instruct-4bit", None)?;
    runner.generate_text(&model_id, &conversation, None, None)?;
    Ok(())
}

fn main() {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();
    let runner = Runner::new();

    match (|| {
        embedding(&runner)?;
        //chat(&runner)?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })() {
        Err(e) => println!("{e}"),
        Ok(_) => (),
    }
}
/*[
"Instruct: Given a web search query, retrieve relevant passages that answer the query\nQuery:What is the capital of China?",
"Instruct: Given a web search query, retrieve relevant passages that answer the query\nQuery:Explain gravity"
]*/
