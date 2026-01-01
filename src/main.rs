use clap::{Parser, Subcommand};

mod db;

#[derive(Parser)]
#[command(name = "engram")]
#[command(about = "Garbage-collected memory for Claude agents")]
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
        /// Scope: global or project:<path>
        #[arg(long, default_value = "global")]
        scope: String,
    },
    /// List memories
    List {
        /// Filter by scope
        #[arg(long)]
        scope: Option<String>,
        /// Filter by generation (0, 1, 2)
        #[arg(long)]
        gen: Option<u8>,
    },
    /// Show a specific memory
    Show {
        /// Memory ID
        id: String,
    },
    /// Edit a memory
    Edit {
        /// Memory ID
        id: String,
    },
    /// Remove a memory
    Remove {
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
    /// Initialize session - output memories for context injection
    Init {
        /// Scopes to include
        #[arg(long)]
        scope: Vec<String>,
    },
    /// Flush session data
    Flush {
        /// Session ID
        #[arg(long)]
        session: String,
    },
    /// Run garbage collection
    Gc {
        /// Dry run - show what would be done
        #[arg(long)]
        dry_run: bool,
        /// Minimum reviews before eligible for GC
        #[arg(long, default_value = "5")]
        min_reviews: u32,
        /// Minimum tap/review ratio to survive (0.0-1.0)
        #[arg(long, default_value = "0.2")]
        min_ratio: f64,
        /// Tap count to promote to next generation
        #[arg(long, default_value = "3")]
        promote_threshold: u32,
    },
    /// Show memory statistics
    Stats,
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
        Commands::Add { content, scope } => {
            match db::add_memory(&conn, &content, &scope) {
                Ok(id) => println!("{}", id),
                Err(e) => {
                    eprintln!("Failed to add memory: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::List { scope, gen } => {
            match db::list_memories(&conn, scope.as_deref(), gen) {
                Ok(memories) => {
                    if memories.is_empty() {
                        println!("No memories found.");
                    } else {
                        for m in memories {
                            println!("[{}] gen{} taps:{} {} | {}", m.id, m.generation, m.tap_count, m.scope, m.content);
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
                    println!("Scope:      {}", m.scope);
                    println!("Generation: {}", m.generation);
                    println!("Taps:       {}", m.tap_count);
                    println!("Reviews:    {}", m.review_count);
                    if m.review_count > 0 {
                        let ratio = (m.tap_count as f64 / m.review_count as f64) * 100.0;
                        println!("Tap ratio:  {:.0}%", ratio);
                    }
                    println!("Created:    {}", m.created_at);
                    if let Some(tapped) = m.last_tapped_at {
                        println!("Last tap:   {}", tapped);
                    }
                    if let Some(reviewed) = m.last_reviewed_at {
                        println!("Last review: {}", reviewed);
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
        Commands::Edit { id } => {
            println!("Edit not yet implemented: {}", id);
        }
        Commands::Remove { id } => {
            match db::remove_memory(&conn, &id) {
                Ok(true) => println!("Removed: {}", id),
                Ok(false) => {
                    eprintln!("Memory not found: {}", id);
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Failed to remove memory: {}", e);
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
        Commands::Init { scope } => {
            match db::get_memories_for_init(&conn, &scope) {
                Ok(memories) => {
                    // Always output context block with instructions
                    println!("<engram-context>");
                    println!("# Engram Memory System");
                    println!();
                    println!("When you learn something worth remembering about this project, store it:");
                    println!("```bash");
                    println!("ENGRAM_DB_PATH=./engram.db ./target/debug/engram add \"<fact>\" --scope \"project:$PWD\"");
                    println!("```");
                    println!();
                    println!("Store: project conventions, user corrections, architecture decisions, gotchas.");
                    println!("Skip: obvious things from code, sensitive info, duplicates of existing memories.");
                    println!();
                    if memories.is_empty() {
                        println!("No memories yet for this project.");
                    } else {
                        println!("## Current Memories");
                        for m in memories {
                            // Include ID as HTML comment for potential tap tracking
                            println!("<!-- {} -->- {}", m.id, m.content);
                        }
                    }
                    println!("</engram-context>");
                }
                Err(e) => {
                    eprintln!("Failed to get memories: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Flush { session } => {
            println!("Flush not yet implemented: {}", session);
        }
        Commands::Gc { dry_run, min_reviews, min_ratio, promote_threshold } => {
            match db::run_gc(&conn, min_reviews, min_ratio, promote_threshold, dry_run) {
                Ok(result) => {
                    let prefix = if dry_run { "[DRY RUN] " } else { "" };

                    if result.expired.is_empty() && result.promoted.is_empty() {
                        println!("{}No changes.", prefix);
                    } else {
                        if !result.expired.is_empty() {
                            println!("{}Expired {} memory(ies):", prefix, result.expired.len());
                            for (id, content, taps, reviews) in &result.expired {
                                let ratio = if *reviews > 0 { *taps as f64 / *reviews as f64 * 100.0 } else { 0.0 };
                                println!("  - {} (taps:{} reviews:{} ratio:{:.0}%)",
                                    truncate(content, 40), taps, reviews, ratio);
                            }
                        }
                        if !result.promoted.is_empty() {
                            println!("{}Promoted {} memory(ies):", prefix, result.promoted.len());
                            for (id, content, taps, reviews) in &result.promoted {
                                println!("  + {} (taps:{})", truncate(content, 40), taps);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to run GC: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Stats => {
            match db::get_stats(&conn) {
                Ok(stats) => {
                    println!("=== Engram Stats ===");
                    println!("Total memories: {}", stats.total);
                    println!();
                    println!("By generation:");
                    println!("  Gen 0 (ephemeral):  {}", stats.by_generation[0]);
                    println!("  Gen 1 (surviving):  {}", stats.by_generation[1]);
                    println!("  Gen 2 (permanent):  {}", stats.by_generation[2]);
                    println!();
                    println!("Taps:");
                    println!("  Total taps:    {}", stats.total_taps);
                    println!("  Never tapped:  {}", stats.never_tapped);
                    println!();
                    if !stats.scopes.is_empty() {
                        println!("By scope:");
                        for (scope, count) in &stats.scopes {
                            println!("  {}: {}", scope, count);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to get stats: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}
