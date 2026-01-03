use clap::{Parser, Subcommand};

mod db;
mod tui;

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
    },
    /// Initialize engram for this project
    Init,
    /// Output prompt snippet for CLAUDE.md
    Prompt,
    /// Launch interactive TUI
    Ui,
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

    let conn = match db::open_db() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to open database: {}", e);
            std::process::exit(1);
        }
    };

    match cli.command {
        Commands::Add { content } => {
            match db::add_memory(&conn, &content) {
                Ok(id) => println!("{}", id),
                Err(e) => {
                    eprintln!("Failed to add memory: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::List { all } => {
            match db::list_memories_filtered(&conn, all) {
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
            match db::get_memory(&conn, &id) {
                Ok(Some(m)) => {
                    println!("ID:         {}", m.id);
                    println!("Content:    {}", m.content);
                    println!("Taps:       {}", m.tap_count);
                    println!("Created:    {}", format_timestamp(m.created_at));
                    if let Some(tapped) = m.last_tapped_at {
                        println!("Last tap:   {}", format_timestamp(tapped));
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
            match db::edit_memory(&conn, &id, &content) {
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
            match db::forget_memory(&conn, &id) {
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
            match db::promote_memory(&conn, &id) {
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
                match db::tap_memories_by_match(&conn, &pattern) {
                    Ok(matched_ids) => tapped.extend(matched_ids),
                    Err(e) => {
                        eprintln!("Failed to tap by match: {}", e);
                        std::process::exit(1);
                    }
                }
            }

            // Tap by explicit IDs
            for id in ids {
                match db::tap_memory(&conn, &id) {
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
        Commands::Log { limit, action, memory } => {
            match db::get_events(&conn, limit, action.as_deref(), memory.as_deref()) {
                Ok(events) => {
                    if events.is_empty() {
                        println!("No events found.");
                    } else {
                        for e in events {
                            let time = format_timestamp(e.timestamp);
                            let mem_id = e.memory_id.as_deref().unwrap_or("-");
                            let short_id = if mem_id.len() > 8 { &mem_id[..8] } else { mem_id };
                            print!("{} {:8} {}", time, e.action, short_id);
                            if let Some(data) = &e.data {
                                print!(" {}", truncate(data, 50));
                            }
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
        Commands::Prompt => {
            print!("{}", include_str!("AGENT_INSTRUCTIONS.md"));
        }
        Commands::Ui => {
            if let Err(e) = tui::run() {
                eprintln!("TUI error: {}", e);
                std::process::exit(1);
            }
        }
    }
}

fn format_timestamp(ts: db::Timestamp) -> String {
    use std::time::{Duration, UNIX_EPOCH};
    let dt = UNIX_EPOCH + Duration::from_secs(ts as u64);
    let datetime: chrono::DateTime<chrono::Local> = dt.into();
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}
