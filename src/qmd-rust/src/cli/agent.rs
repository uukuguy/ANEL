use crate::cli::AgentArgs;
use crate::llm::Router;
use crate::store::Store;
use anyhow::Result;
use dialoguer::Input;

/// Handle agent command - autonomous search mode
pub fn handle(
    cmd: &AgentArgs,
    store: &Store,
    llm: &Router,
) -> Result<()> {
    if cmd.interactive {
        run_interactive_agent(store, llm)?;
    } else {
        // Single query mode
        let query: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Enter search query")
            .interact_text()?;

        println!("Processing query: {}", query);

        // TODO: Implement intelligent query classification and routing
        // - keyword queries → BM25
        // - semantic queries → vector search
        // - complex queries → hybrid search

        println!("Query processed");
    }

    Ok(())
}

fn run_interactive_agent(_store: &Store, _llm: &Router) -> Result<()> {
    println!("QMD Agent Mode - Interactive");
    println!("Type 'exit' to quit, 'help' for commands");

    loop {
        let query: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("qmd>")
            .interact_text()?;

        if query.trim().eq_ignore_ascii_case("exit") {
            break;
        }

        if query.trim().is_empty() {
            continue;
        }

        // TODO: Implement agent logic
        println!("Agent processing: {}", query);
    }

    Ok(())
}
