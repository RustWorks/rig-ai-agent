use std::env;

use rig::{
    completion::Prompt,
    embeddings::{builder::DocumentEmbeddings, builder::EmbeddingsBuilder},
    providers::openai::{Client, TEXT_EMBEDDING_ADA_002},
    vector_store::in_memory_store::InMemoryVectorStore,
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Create OpenAI client
    let openai_api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    let openai_client = Client::new(&openai_api_key);

    let embedding_model = openai_client.embedding_model(TEXT_EMBEDDING_ADA_002);

    let embeddings = EmbeddingsBuilder::new(embedding_model.clone())
        .simple_document("doc0", "Definition of a *flurbo*: A flurbo is a green alien that lives on cold planets")
        .simple_document("doc1", "Definition of a *glarb-glarb*: A glarb-glarb is a ancient tool used by the ancestors of the inhabitants of planet Jiro to farm the land.")
        .simple_document("doc2", "Definition of a *linglingdong*: A term used by inhabitants of the far side of the moon to describe humans.")
        .build()
        .await?;

    let index = InMemoryVectorStore::default()
        .add_documents(
            embeddings
                .into_iter()
                .map(
                    |DocumentEmbeddings {
                         id,
                         document,
                         embeddings,
                     }| { (id, document, embeddings) },
                )
                .collect(),
        )?
        .index(embedding_model);

    let rag_agent = openai_client.agent("gpt-4")
        .preamble("
            You are a dictionary assistant here to assist the user in understanding the meaning of words.
            You will find additional non-standard word definitions that could be useful below.
        ")
        .dynamic_context(1, index)
        .build();

    // Prompt the agent and print the response
    let response = rag_agent.prompt("What does \"glarb-glarb\" mean?").await?;

    println!("{}", response);

    Ok(())
}
