use clap::{Parser, Subcommand};

mod db;
mod engram;
mod mitm;
mod tui;

use engram::Engram;

#[derive(Parser)]
#[command(name = "engram")]
#[command(about = "Memory observability for Claude agents")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new memory
    Add {
        /// The memory content
        content: String,
    },
    /// List memories
    List {
        /// Include promoted/forgotten memories
        #[arg(long, short)]
        all: bool,
    },
    /// Show a specific memory
    Show {
        /// Memory ID
        id: String,
    },
    /// Edit a memory's content
    Edit {
        /// Memory ID
        id: String,
        /// New content
        content: String,
    },
    /// Forget a memory (mark as discarded)
    Forget {
        /// Memory ID
        id: String,
    },
    /// Promote a memory to CLAUDE.md
    Promote {
        /// Memory ID
        id: String,
    },
    /// Record memory usage (tap)
    Tap {
        /// Memory IDs to tap
        ids: Vec<String>,
        /// Match memories by substring
        #[arg(long = "match")]
        match_str: Option<String>,
    },
    /// View event log
    Log {
        /// Number of events to show
        #[arg(long, short, default_value = "20")]
        limit: u32,
        /// Filter by action (ADD, TAP, REMOVE, EDIT)
        #[arg(long)]
        action: Option<String>,
        /// Filter by memory ID
        #[arg(long)]
        memory: Option<String>,
        /// Include TAP events for promoted memories
        #[arg(long)]
        all: bool,
    },
    /// Initialize engram for this project
    Init,
    /// Output agent instructions for context recovery
    Prime,
    /// Launch interactive TUI
    Ui,
    /// Watch mitmproxy for Claude API traffic and extract memories
    Mitmproxy {
        /// Mitmproxy web API URL
        #[arg(long, default_value = "http://localhost:8081")]
        url: String,
        /// Poll interval in seconds
        #[arg(long, short, default_value = "5")]
        interval: u64,
        /// Batch size before running analyzer
        #[arg(long, short, default_value = "10")]
        batch: usize,
    },
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

fn main() {
    let cli = Cli::parse();

    let engram = match Engram::from_env() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Failed to open database: {}", e);
            std::process::exit(1);
        }
    };

    match cli.command {
        Commands::Add { content } => {
            match engram.add_memory(&content) {
                Ok(id) => println!("{}", id),
                Err(e) => {
                    eprintln!("Failed to add memory: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::List { all } => {
            match engram.list_memories_filtered(all) {
                Ok(memories) => {
                    if memories.is_empty() {
                        println!("No memories found.");
                    } else {
                        for m in memories {
                            println!("[{}] taps:{} | {}", m.id, m.tap_count, m.content);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to list memories: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Show { id } => {
            match engram.get_memory(&id) {
                Ok(Some(m)) => {
                    println!("ID:         {}", m.id);
                    println!("Content:    {}", m.content);
                    println!("Taps:       {}", m.tap_count);
                    println!("Created:    {}", m.created_at);
                    if let Some(ref tapped) = m.last_tapped_at {
                        println!("Last tap:   {}", tapped);
                    }
                }
                Ok(None) => {
                    eprintln!("Memory not found: {}", id);
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Failed to get memory: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Edit { id, content } => {
            match engram.edit_memory(&id, &content) {
                Ok(true) => println!("Updated: {}", id),
                Ok(false) => {
                    eprintln!("Memory not found: {}", id);
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Failed to edit memory: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Forget { id } => {
            match engram.forget_memory(&id) {
                Ok(true) => println!("Forgotten: {}", id),
                Ok(false) => {
                    eprintln!("Memory not found: {}", id);
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Failed to forget memory: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Promote { id } => {
            match engram.promote_memory(&id) {
                Ok(Some(content)) => {
                    // Output markdown format for CLAUDE.md
                    println!("- {}", content);
                }
                Ok(None) => {
                    eprintln!("Memory not found: {}", id);
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Failed to promote memory: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Tap { ids, match_str } => {
            let mut tapped = Vec::new();
            let mut not_found = Vec::new();

            // Tap by match pattern first
            if let Some(pattern) = match_str {
                match engram.tap_memories_by_match(&pattern) {
                    Ok(matched_ids) => tapped.extend(matched_ids),
                    Err(e) => {
                        eprintln!("Failed to tap by match: {}", e);
                        std::process::exit(1);
                    }
                }
            }

            // Tap by explicit IDs
            for id in ids {
                match engram.tap_memory(&id) {
                    Ok(true) => tapped.push(id),
                    Ok(false) => not_found.push(id),
                    Err(e) => {
                        eprintln!("Failed to tap {}: {}", id, e);
                        std::process::exit(1);
                    }
                }
            }

            if tapped.is_empty() && not_found.is_empty() {
                println!("No memories to tap.");
            } else {
                if !tapped.is_empty() {
                    println!("Tapped {} memory(ies): {}", tapped.len(), tapped.join(", "));
                }
                if !not_found.is_empty() {
                    eprintln!("Not found: {}", not_found.join(", "));
                    std::process::exit(1);
                }
            }
        }
        Commands::Log { limit, action, memory, all } => {
            match engram.get_enriched_events(limit, action.as_deref(), memory.as_deref(), all) {
                Ok(events) => {
                    if events.is_empty() {
                        println!("No events found.");
                    } else {
                        for e in events {
                            let mem_id = e.memory_id.as_deref().unwrap_or("-");
                            let short_id = if mem_id.len() > 8 { &mem_id[..8] } else { mem_id };
                            print!("{} {:8} {}", e.timestamp, e.action, short_id);
                            print!(" {}", truncate(&e.content, 50));
                            println!();
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to get events: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Init => {
            // DB is already opened/created above, just confirm
            println!("Initialized engram in .engram/");
        }
        Commands::Prime => {
            print!("{}", include_str!("AGENT_INSTRUCTIONS.md"));
        }
        Commands::Ui => {
            if let Err(e) = tui::run() {
                eprintln!("TUI error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Mitmproxy { url, interval, batch } => {
            // Create tokio runtime for async operations
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                if let Err(e) = mitm::run_watcher(&url, interval, batch, engram).await {
                    eprintln!("Mitmproxy watcher error: {}", e);
                    std::process::exit(1);
                }
            });
        }
    }
}

