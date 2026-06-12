use clap::{Parser, Subcommand};
use anyhow::Result;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "stateforge", about = "State machine generator and visualizer", version)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Parse a source file and extract a state machine
    Parse {
        /// Source file path (Swift, Kotlin, TypeScript, Go)
        file: String,
        /// Output format: mermaid, graphviz, svg, json
        #[arg(short, long, default_value = "mermaid")]
        format: String,
        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Analyze log files for event sequences
    Logs {
        /// Log file path
        file: String,
        #[arg(short, long, default_value = "mermaid")]
        format: String,
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Generate code from a saved state machine
    Generate {
        /// State machine ID or name
        id: String,
        /// Target language: swift, kotlin, typescript, go
        #[arg(short, long, default_value = "typescript")]
        language: String,
        #[arg(short, long)]
        output: Option<String>,
    },
    /// List all saved state machines
    List,
    /// Use AI to extract a state machine from a text description
    Describe {
        /// Description text
        description: String,
        #[arg(short, long, default_value = "mermaid")]
        format: String,
    },
    /// Configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    SetKey { key: String },
    Check,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("stateforge=info".parse()?))
        .with_target(false)
        .init();

    let cli = Cli::parse();
    let db_path = get_db_path();
    std::fs::create_dir_all(db_path.parent().unwrap())?;

    match cli.command {
        Cmd::Parse { file, format, output } => {
            cmd_parse(&file, &format, output.as_deref(), &db_path).await?;
        }
        Cmd::Logs { file, format, output } => {
            cmd_logs(&file, &format, output.as_deref()).await?;
        }
        Cmd::Generate { id, language, output } => {
            cmd_generate(&id, &language, output.as_deref(), &db_path).await?;
        }
        Cmd::List => {
            cmd_list(&db_path).await?;
        }
        Cmd::Describe { description, format } => {
            cmd_describe(&description, &format).await?;
        }
        Cmd::Config { action } => {
            cmd_config(action).await?;
        }
    }
    Ok(())
}

fn get_db_path() -> std::path::PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("stateforge")
        .join("stateforge.db")
}

async fn cmd_parse(file: &str, format: &str, output: Option<&str>, db_path: &std::path::Path) -> Result<()> {
    let content = std::fs::read_to_string(file)?;
    let mut sm = sf_core::parser::parse_file(file, &content)?;

    let pool = sf_core::db::open(db_path).await?;
    sf_core::db::queries::insert_machine(&pool, &sm).await?;

    let config = make_config(format);
    let diagram = sf_core::diagram::render(&sm, &config)?;

    println!("State machine: {} ({} states, {} transitions)",
        sm.name, sm.states.len(), sm.transitions.len());

    write_or_print(&diagram, output)?;
    Ok(())
}

async fn cmd_logs(file: &str, format: &str, output: Option<&str>) -> Result<()> {
    let content = std::fs::read_to_string(file)?;
    let sm = sf_core::log_analyzer::LogAnalyzer::analyze(&content, Some(file))?;

    println!("Flow: {} ({} states, {} transitions)",
        sm.name, sm.states.len(), sm.transitions.len());

    let config = make_config(format);
    let diagram = sf_core::diagram::render(&sm, &config)?;
    write_or_print(&diagram, output)?;
    Ok(())
}

async fn cmd_generate(id: &str, language: &str, output: Option<&str>, db_path: &std::path::Path) -> Result<()> {
    use sf_core::models::Language;

    let pool = sf_core::db::open(db_path).await?;
    let machines = sf_core::db::queries::list_machines(&pool).await?;
    let sm = machines.into_iter()
        .find(|m| m.id == id || m.name.to_lowercase().contains(&id.to_lowercase()))
        .ok_or_else(|| anyhow::anyhow!("Machine not found: {}", id))?;

    let lang = match language {
        "swift"      => Language::Swift,
        "kotlin"     => Language::Kotlin,
        "go"         => Language::Go,
        _            => Language::TypeScript,
    };

    let code = sf_core::generator::generate(&sm, &lang)?;
    write_or_print(&code, output)?;
    Ok(())
}

async fn cmd_list(db_path: &std::path::Path) -> Result<()> {
    let pool = sf_core::db::open(db_path).await?;
    let machines = sf_core::db::queries::list_machines(&pool).await?;

    if machines.is_empty() {
        println!("No saved state machines.");
        return Ok(());
    }

    println!("{:<36}  {:<30}  {:<6}  {:<6}",
        "ID", "Name", "States", "Trans");
    println!("{}", "-".repeat(84));
    for m in machines {
        println!("{:<36}  {:<30}  {:<6}  {:<6}",
            &m.id[..8.min(m.id.len())], m.name, m.states.len(), m.transitions.len());
    }
    Ok(())
}

async fn cmd_describe(description: &str, format: &str) -> Result<()> {
    let api_key = get_api_key()?;
    let analyzer = sf_core::ai::claude::ClaudeAnalyzer::new(api_key);
    let sm = sf_core::ai::AiAnalyzer::extract_from_description(&analyzer, description).await?;

    println!("Extracted: {} ({} states, {} transitions)",
        sm.name, sm.states.len(), sm.transitions.len());

    let config = make_config(format);
    let diagram = sf_core::diagram::render(&sm, &config)?;
    println!("{}", diagram);
    Ok(())
}

async fn cmd_config(action: ConfigAction) -> Result<()> {
    match action {
        ConfigAction::SetKey { key } => {
            keyring::Entry::new("stateforge", "claude_api_key")?.set_password(&key)?;
            println!("API key stored.");
        }
        ConfigAction::Check => {
            match get_api_key() {
                Ok(_) => println!("API key: set"),
                Err(_) => println!("API key: not set  (run `stateforge config set-key sk-ant-...`)"),
            }
        }
    }
    Ok(())
}

fn make_config(format: &str) -> sf_core::models::diagram::DiagramConfig {
    use sf_core::models::diagram::{DiagramConfig, DiagramFormat};
    let fmt = match format {
        "graphviz" | "dot" => DiagramFormat::GraphvizDot,
        "svg"              => DiagramFormat::Svg,
        "json"             => DiagramFormat::Json,
        "sequence"         => DiagramFormat::MermaidSequence,
        "flowchart"        => DiagramFormat::MermaidFlowchart,
        _                  => DiagramFormat::MermaidState,
    };
    DiagramConfig { format: fmt, ..DiagramConfig::default() }
}

fn write_or_print(content: &str, output: Option<&str>) -> Result<()> {
    if let Some(path) = output {
        std::fs::write(path, content)?;
        println!("Written to {}", path);
    } else {
        println!("{}", content);
    }
    Ok(())
}

fn get_api_key() -> Result<String> {
    let kr = keyring::Entry::new("stateforge", "claude_api_key")?;
    Ok(kr.get_password()?)
}
