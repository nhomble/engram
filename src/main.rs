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
    /// Edit a memory's content
    Edit {
        /// Memory ID
        id: String,
        /// New content
        content: String,
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
    /// Run garbage collection
    Gc {
        /// Dry run - show what would be done
        #[arg(long)]
        dry_run: bool,
        /// Minimum taps to survive GC (memories with fewer taps are expired)
        #[arg(long, default_value = "1")]
        min_taps: u32,
        /// Tap count to promote to next generation
        #[arg(long, default_value = "3")]
        promote_threshold: u32,
    },
    /// Show memory statistics
    Stats,
    /// View event log
    Log {
        /// Number of events to show
        #[arg(long, short, default_value = "20")]
        limit: u32,
        /// Filter by action (ADD, TAP, REMOVE, REVIEW, EXPIRE, PROMOTE)
        #[arg(long)]
        action: Option<String>,
        /// Filter by memory ID
        #[arg(long)]
        memory: Option<String>,
    },
    /// Show hot memories (most tapped recently)
    Hot {
        /// Time window in hours (default: 24)
        #[arg(long, short, default_value = "24")]
        hours: u32,
        /// Number of memories to show
        #[arg(long, short, default_value = "10")]
        limit: u32,
    },
    /// Show activity summary by day
    Activity {
        /// Number of days to show (default: 7)
        #[arg(long, short, default_value = "7")]
        days: u32,
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
        Commands::Gc { dry_run, min_taps, promote_threshold } => {
            match db::run_gc(&conn, min_taps, promote_threshold, dry_run) {
                Ok(result) => {
                    let prefix = if dry_run { "[DRY RUN] " } else { "" };

                    if result.expired.is_empty() && result.promoted.is_empty() {
                        println!("{}No changes.", prefix);
                    } else {
                        if !result.expired.is_empty() {
                            println!("{}Expired {} memory(ies):", prefix, result.expired.len());
                            for (_id, content, taps) in &result.expired {
                                println!("  - {} (taps:{})", truncate(content, 40), taps);
                            }
                        }
                        if !result.promoted.is_empty() {
                            println!("{}Promoted {} memory(ies):", prefix, result.promoted.len());
                            for (_id, content, taps) in &result.promoted {
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
        Commands::Hot { hours, limit } => {
            let window_secs = hours as i64 * 3600;
            match db::get_hot_memories(&conn, window_secs, limit) {
                Ok(memories) => {
                    if memories.is_empty() {
                        println!("No hot memories in the last {} hours.", hours);
                    } else {
                        println!("=== Hot Memories (last {} hours) ===", hours);
                        for m in memories {
                            println!("[{}] {} taps ({}x recent) | {}",
                                &m.id[..8.min(m.id.len())],
                                m.total_taps,
                                m.recent_taps,
                                truncate(&m.content, 50));
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to get hot memories: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Activity { days } => {
            match db::get_activity_by_day(&conn, days) {
                Ok(activity) => {
                    if activity.is_empty() {
                        println!("No activity in the last {} days.", days);
                    } else {
                        println!("=== Activity (last {} days) ===", days);
                        println!("{:>12}  {:>4}  {:>4}  {:>4}  {:>6}", "Date", "Add", "Tap", "Rem", "Review");
                        println!("{}", "-".repeat(42));
                        for a in activity {
                            println!("{:>12}  {:>4}  {:>4}  {:>4}  {:>6}",
                                a.period, a.adds, a.taps, a.removes, a.reviews);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to get activity: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}

fn format_timestamp(ts: i64) -> String {
    use std::time::{Duration, UNIX_EPOCH};
    let dt = UNIX_EPOCH + Duration::from_secs(ts as u64);
    let datetime: chrono::DateTime<chrono::Local> = dt.into();
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}
