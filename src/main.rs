mod db;
mod engine;
mod error;
mod sql;
mod storage;
mod transaction;
mod types;
mod mvcc;
mod document;
mod vector;
mod search;
mod crypto;
mod migration;
mod backup;
mod export;

use std::path::PathBuf;
use std::io::{self, Write};

use clap::{Parser, Subcommand};
use colored::*;

use db::Database;
use engine::QueryResult;

#[derive(Parser)]
#[command(name = "tywindb")]
#[command(about = "A modern, fast, and easy-to-use database")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Start interactive REPL")]
    Repl {
        #[arg(short, long, default_value = "tywindb.tdb")]
        db: PathBuf,
    },

    #[command(about = "Execute SQL from a file")]
    Exec {
        #[arg(short, long, default_value = "tywindb.tdb")]
        db: PathBuf,

        file: PathBuf,
    },

    #[command(about = "Run SQL query directly")]
    Query {
        #[arg(short, long, default_value = "tywindb.tdb")]
        db: PathBuf,

        #[arg(short, long)]
        sql: String,
    },

    #[command(about = "Set database password")]
    Passwd {
        #[arg(short, long, default_value = "tywindb.tdb")]
        db: PathBuf,
    },

    #[command(about = "Backup database")]
    Backup {
        #[arg(short, long, default_value = "tywindb.tdb")]
        db: PathBuf,

        #[arg(short, long)]
        output: PathBuf,

        #[arg(short, long, default_value = "true")]
        compress: bool,
    },

    #[command(about = "Restore database from backup")]
    Restore {
        #[arg(short, long, default_value = "tywindb.tdb")]
        db: PathBuf,

        #[arg(short, long)]
        backup: PathBuf,
    },

    #[command(about = "Create migration")]
    MigrateCreate {
        #[arg(short, long, default_value = "tywindb.tdb")]
        db: PathBuf,

        name: String,
    },

    #[command(about = "Run pending migrations")]
    MigrateUp {
        #[arg(short, long, default_value = "tywindb.tdb")]
        db: PathBuf,
    },

    #[command(about = "Rollback last migration")]
    MigrateDown {
        #[arg(short, long, default_value = "tywindb.tdb")]
        db: PathBuf,
    },

    #[command(about = "Show migration status")]
    MigrateStatus {
        #[arg(short, long, default_value = "tywindb.tdb")]
        db: PathBuf,
    },

    #[command(about = "Rollback to specific version")]
    MigrateRollback {
        #[arg(short, long, default_value = "tywindb.tdb")]
        db: PathBuf,

        version: u32,
    },

    #[command(about = "Dry run migration (preview SQL)")]
    MigrateDryRun {
        #[arg(short, long, default_value = "tywindb.tdb")]
        db: PathBuf,

        version: u32,
    },

    #[command(about = "Export table to CSV")]
    ExportCsv {
        #[arg(short, long, default_value = "tywindb.tdb")]
        db: PathBuf,

        table: String,

        #[arg(short, long)]
        output: PathBuf,
    },

    #[command(about = "Import CSV to table")]
    ImportCsv {
        #[arg(short, long, default_value = "tywindb.tdb")]
        db: PathBuf,

        table: String,

        #[arg(short, long)]
        input: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Repl { db } => run_repl(db),
        Commands::Exec { db, file } => run_exec(db, file),
        Commands::Query { db, sql } => run_query(db, &sql),
        Commands::Passwd { db } => run_passwd(db),
        Commands::Backup { db, output, compress } => run_backup(db, output, compress),
        Commands::Restore { db, backup } => run_restore(db, backup),
        Commands::MigrateCreate { db, name } => run_migrate_create(db, &name),
        Commands::MigrateUp { db } => run_migrate_up(db),
        Commands::MigrateDown { db } => run_migrate_down(db),
        Commands::MigrateStatus { db } => run_migrate_status(db),
        Commands::MigrateRollback { db, version } => run_migrate_rollback(db, version),
        Commands::MigrateDryRun { db, version } => run_migrate_dry_run(db, version),
        Commands::ExportCsv { db, table, output } => run_export_csv(db, &table, output),
        Commands::ImportCsv { db, table, input } => run_import_csv(db, &table, input),
    }
}

fn prompt_password(prompt: &str) -> Result<String, io::Error> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    Ok(password.trim().to_string())
}

fn print_result(result: &QueryResult) {
    match result {
        QueryResult::Rows(rows) => {
            if rows.is_empty() {
                println!("{}", "Empty set".dimmed());
                return;
            }

            if let Some(first_row) = rows.first() {
                let cols: Vec<&String> = first_row.keys().collect();
                let col_widths: Vec<usize> = cols.iter().map(|c| {
                    let header_len = c.len();
                    let max_val_len = rows.iter()
                        .map(|r| r.get(*c).map(|v| v.to_string().len()).unwrap_or(4))
                        .max()
                        .unwrap_or(0);
                    header_len.max(max_val_len)
                }).collect();

                let total_width: usize = col_widths.iter().sum::<usize>() + 3 * (cols.len() - 1);

                println!("{}", "─".repeat(total_width).dimmed());

                let header: Vec<String> = cols.iter().enumerate().map(|(i, c)| {
                    format!("{:>width$}", c.white().bold(), width = col_widths[i])
                }).collect();
                println!("{}", header.join(" │ "));

                println!("{}", "─".repeat(total_width).dimmed());

                for row in rows {
                    let values: Vec<String> = cols.iter().enumerate().map(|(i, c)| {
                        match row.get(*c) {
                            Some(val) => {
                                let s = val.to_string();
                                let formatted = if val.is_null() {
                                    "NULL".dimmed().italic().to_string()
                                } else if s.starts_with('\'') || s.starts_with('"') {
                                    s.green().to_string()
                                } else if s.parse::<i64>().is_ok() || s.parse::<f64>().is_ok() {
                                    s.yellow().to_string()
                                } else if s == "true" || s == "false" {
                                    s.cyan().to_string()
                                } else {
                                    s.white().to_string()
                                };
                                format!("{:>width$}", formatted, width = col_widths[i])
                            }
                            None => format!("{:>width$}", "NULL".dimmed().italic(), width = col_widths[i]),
                        }
                    }).collect();
                    println!("  {}", values.join(" │ "));
                }

                println!("{}", "─".repeat(total_width).dimmed());
                println!("{} {}", rows.len().to_string().green().bold(), "rows".dimmed());
            }
        }
        QueryResult::RowsAffected(n) => {
            println!("{} {}", "Query OK,".green(), format!("{} rows affected", n).cyan());
        }
        QueryResult::TableCreated => {
            println!("{}", "Table created".green());
        }
        QueryResult::TransactionStarted { tx_id } => {
            println!("{} {}", "Transaction".cyan(), format!("#{}", tx_id).cyan().bold());
        }
        QueryResult::TransactionCommitted => {
            println!("{}", "Committed".green());
        }
        QueryResult::TransactionRolledBack => {
            println!("{}", "Rolled back".yellow());
        }
        QueryResult::Empty => {
            println!("{}", "OK".green());
        }
    }
}

fn run_repl(db_path: PathBuf) -> anyhow::Result<()> {
    let mut db = Database::open(&db_path)?;

    if db.has_password() {
        println!("{}", "Database is password protected.".yellow().bold());
        let password = prompt_password("Password: ")?;
        if !db.authenticate(&password)? {
            eprintln!("{}", "Authentication failed!".red().bold());
            return Ok(());
        }
        println!("{}", "Authenticated.".green());
    }

    println!("\n{}", "tywindb v0.1.0".cyan().bold());
    println!("Database: {}\n", db_path.display().to_string().white());
    println!("Type {} for help, {} to quit.\n", "help".green(), "exit".green());

    loop {
        print!("{} ", "tywindb".green().bold());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        match input.to_lowercase().as_str() {
            "exit" | "quit" => {
                println!("{}", "Bye!".green());
                break;
            }
            "help" => {
                println!("\n{}", "Commands:".yellow().bold());
                println!("  help      Show this help");
                println!("  passwd    Set database password");
                println!("  exit      Quit the REPL");
                println!();
                println!("{}", "SQL:".yellow().bold());
                println!("  SELECT * FROM table [WHERE condition]");
                println!("  INSERT INTO table (col1, col2) VALUES (v1, v2)");
                println!("  UPDATE table SET col1 = value [WHERE condition]");
                println!("  DELETE FROM table [WHERE condition]");
                println!("  CREATE TABLE table (col1 TYPE, col2 TYPE)");
                println!("  BEGIN / COMMIT / ROLLBACK\n");
            }
            "passwd" => {
                let new_pass = prompt_password("New password: ")?;
                let confirm = prompt_password("Confirm password: ")?;
                if new_pass == confirm {
                    db.set_password(&new_pass)?;
                    println!("{}", "Password set.".green());
                } else {
                    eprintln!("{}", "Passwords don't match!".red());
                }
            }
            _ => {
                match db.query(input) {
                    Ok(result) => print_result(&result),
                    Err(e) => eprintln!("{} {}", "Error:".red().bold(), e),
                }
                println!();
            }
        }
    }

    Ok(())
}

fn run_exec(db_path: PathBuf, file_path: PathBuf) -> anyhow::Result<()> {
    let mut db = Database::open(&db_path)?;

    if db.has_password() {
        let password = prompt_password("Password: ")?;
        if !db.authenticate(&password)? {
            eprintln!("{}", "Authentication failed!".red().bold());
            return Ok(());
        }
    }

    let sql = std::fs::read_to_string(&file_path)?;

    for stmt in sql.split(';') {
        let stmt = stmt.trim();
        if stmt.is_empty() {
            continue;
        }

        match db.query(stmt) {
            Ok(result) => print_result(&result),
            Err(e) => eprintln!("{} {}", "Error:".red().bold(), e),
        }
    }

    Ok(())
}

fn run_query(db_path: PathBuf, sql: &str) -> anyhow::Result<()> {
    let mut db = Database::open(&db_path)?;

    if db.has_password() {
        let password = prompt_password("Password: ")?;
        if !db.authenticate(&password)? {
            eprintln!("{}", "Authentication failed!".red().bold());
            return Ok(());
        }
    }

    let result = db.query(sql)?;
    print_result(&result);
    Ok(())
}

fn run_passwd(db_path: PathBuf) -> anyhow::Result<()> {
    let mut db = Database::open(&db_path)?;

    if db.has_password() {
        let old_pass = prompt_password("Current password: ")?;
        if !db.authenticate(&old_pass)? {
            eprintln!("{}", "Authentication failed!".red().bold());
            return Ok(());
        }
    }

    let new_pass = prompt_password("New password: ")?;
    let confirm = prompt_password("Confirm password: ")?;

    if new_pass == confirm {
        db.set_password(&new_pass)?;
        println!("{}", "Password set.".green());
    } else {
        eprintln!("{}", "Passwords don't match!".red());
    }

    Ok(())
}

fn run_backup(db_path: PathBuf, output: PathBuf, compress: bool) -> anyhow::Result<()> {
    let mut db = Database::open(&db_path)?;

    if db.has_password() {
        let password = prompt_password("Password: ")?;
        if !db.authenticate(&password)? {
            eprintln!("{}", "Authentication failed!".red().bold());
            return Ok(());
        }
    }

    let data_dir = if db_path.is_dir() {
        db_path.clone()
    } else {
        db_path.parent().unwrap_or(&PathBuf::from(".")).to_path_buf()
    };
    let manager = backup::BackupManager::new(&data_dir);

    let info = manager.backup(&output, compress)?;

    println!("{} {}", "Backup created:".green(), info.filename.cyan());
    println!("  {} {}", "Size:".dimmed(), info.size);
    println!("  {} {}", "Compressed:".dimmed(), info.compressed);

    Ok(())
}

fn run_restore(db_path: PathBuf, backup_path: PathBuf) -> anyhow::Result<()> {
    let mut db = Database::open(&db_path)?;

    if db.has_password() {
        let password = prompt_password("Current password: ")?;
        if !db.authenticate(&password)? {
            eprintln!("{}", "Authentication failed!".red().bold());
            return Ok(());
        }
    }

    let data_dir = if db_path.is_dir() {
        db_path.clone()
    } else {
        db_path.parent().unwrap_or(&PathBuf::from(".")).to_path_buf()
    };
    let manager = backup::BackupManager::new(&data_dir);

    let new_password = prompt_password("New password (empty to skip): ")?;
    let password = if new_password.is_empty() { None } else { Some(new_password.as_str()) };

    manager.restore(&backup_path, password)?;

    println!("{}", "Database restored.".green());

    Ok(())
}

fn run_migrate_create(db_path: PathBuf, name: &str) -> anyhow::Result<()> {
    let data_dir = if db_path.is_dir() {
        db_path.clone()
    } else {
        db_path.parent().unwrap_or(&PathBuf::from(".")).to_path_buf()
    };
    let manager = migration::MigrationManager::new(&data_dir);

    manager.init()?;

    let up = format!("-- Your migration up SQL here\n-- CREATE TABLE example (id INTEGER, name TEXT)");
    let down = format!("-- Your migration down SQL here\n-- DROP TABLE example");

    let path = manager.create(name, &up, &down)?;

    println!("{} {}", "Migration created:".green(), path.display().to_string().cyan());

    Ok(())
}

fn run_migrate_up(db_path: PathBuf) -> anyhow::Result<()> {
    let mut db = Database::open(&db_path)?;

    if db.has_password() {
        let password = prompt_password("Password: ")?;
        if !db.authenticate(&password)? {
            eprintln!("{}", "Authentication failed!".red().bold());
            return Ok(());
        }
    }

    let data_dir = if db_path.is_dir() {
        db_path.clone()
    } else {
        db_path.parent().unwrap_or(&PathBuf::from(".")).to_path_buf()
    };
    let manager = migration::MigrationManager::new(&data_dir);

    let result = db.query("SELECT version FROM _migrations ORDER BY version")?;
    let applied: Vec<u32> = match &result {
        QueryResult::Rows(rows) => rows.iter()
            .filter_map(|r| r.get("version").and_then(|v| v.as_i64()).map(|v| v as u32))
            .collect(),
        _ => Vec::new(),
    };

    let pending = manager.get_pending(&applied)?;

    if pending.is_empty() {
        println!("{}", "No pending migrations.".yellow());
        return Ok(());
    }

    for m in &pending {
        println!("{} {}", "Running migration:".cyan(), m.name);
        db.query(&m.up)?;
        db.query(&format!("INSERT INTO _migrations (version, name) VALUES ({}, '{}')", m.version, m.name))?;
        println!("  {}", "Done.".green());
    }

    println!("{} {} migrations ran.", "Success:".green(), pending.len());

    Ok(())
}

fn run_migrate_down(db_path: PathBuf) -> anyhow::Result<()> {
    let mut db = Database::open(&db_path)?;

    if db.has_password() {
        let password = prompt_password("Password: ")?;
        if !db.authenticate(&password)? {
            eprintln!("{}", "Authentication failed!".red().bold());
            return Ok(());
        }
    }

    let data_dir = if db_path.is_dir() {
        db_path.clone()
    } else {
        db_path.parent().unwrap_or(&PathBuf::from(".")).to_path_buf()
    };
    let manager = migration::MigrationManager::new(&data_dir);

    let result = db.query("SELECT version, name FROM _migrations ORDER BY version DESC LIMIT 1")?;
    match &result {
        QueryResult::Rows(rows) if !rows.is_empty() => {
            let version = rows[0].get("version").and_then(|v| v.as_i64()).unwrap_or(0) as u32;
            let name = rows[0].get("name").and_then(|v| v.as_str()).unwrap_or("");

            let files = manager.list_files()?;
            let filename = files.iter().find(|f| f.starts_with(&format!("{:04}_", version)));

            if let Some(filename) = filename {
                let content = std::fs::read_to_string(data_dir.join("migrations").join(filename))?;
                let (_, down) = migration::MigrationManager::new(&data_dir).get_pending(&[]).unwrap_or_default()
                    .into_iter()
                    .find(|m| m.version == version)
                    .map(|m| (m.up, m.down))
                    .unwrap_or((String::new(), String::new()));

                if !down.is_empty() {
                    println!("{} {}", "Rolling back migration:".yellow(), name);
                    db.query(&down)?;
                    db.query(&format!("DELETE FROM _migrations WHERE version = {}", version))?;
                    println!("  {}", "Done.".green());
                }
            }
        }
        _ => {
            println!("{}", "No migrations to rollback.".yellow());
        }
    }

    Ok(())
}

fn run_export_csv(db_path: PathBuf, table: &str, output: PathBuf) -> anyhow::Result<()> {
    let mut db = Database::open(&db_path)?;

    if db.has_password() {
        let password = prompt_password("Password: ")?;
        if !db.authenticate(&password)? {
            eprintln!("{}", "Authentication failed!".red().bold());
            return Ok(());
        }
    }

    let result = db.query(&format!("SELECT * FROM {}", table))?;
    match result {
        QueryResult::Rows(rows) => {
            export::ExportManager::to_csv(&rows, &output)?;
            println!("{} {} rows exported to {}", "Exported:".green(), rows.len(), output.display());
        }
        _ => {
            eprintln!("{}", "No data to export.".red());
        }
    }

    Ok(())
}

fn run_import_csv(db_path: PathBuf, table: &str, input: PathBuf) -> anyhow::Result<()> {
    let mut db = Database::open(&db_path)?;

    if db.has_password() {
        let password = prompt_password("Password: ")?;
        if !db.authenticate(&password)? {
            eprintln!("{}", "Authentication failed!".red().bold());
            return Ok(());
        }
    }

    let rows = export::ExportManager::from_csv(&input)?;

    if rows.is_empty() {
        println!("{}", "No data to import.".yellow());
        return Ok(());
    }

    let cols: Vec<&String> = rows[0].keys().collect();
    let col_names = cols.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ");

    for row in &rows {
        let values: Vec<String> = cols.iter().map(|c| {
            match row.get(*c) {
                Some(val) => match val {
                    types::Value::Null => "NULL".to_string(),
                    types::Value::Boolean(b) => b.to_string(),
                    types::Value::Integer(i) => i.to_string(),
                    types::Value::Float(f) => f.to_string(),
                    types::Value::Text(s) => format!("'{}'", s.replace('\'', "''")),
                    _ => format!("'{}'", val.to_string().replace('\'', "''")),
                },
                None => "NULL".to_string(),
            }
        }).collect();

        let sql = format!("INSERT INTO {} ({}) VALUES ({})", table, col_names, values.join(", "));
        db.query(&sql)?;
    }

    println!("{} {} rows imported.", "Imported:".green(), rows.len());

    Ok(())
}

fn run_migrate_status(db_path: PathBuf) -> anyhow::Result<()> {
    let mut db = Database::open(&db_path)?;

    if db.has_password() {
        let password = prompt_password("Password: ")?;
        if !db.authenticate(&password)? {
            eprintln!("{}", "Authentication failed!".red().bold());
            return Ok(());
        }
    }

    let data_dir = if db_path.is_dir() {
        db_path.clone()
    } else {
        db_path.parent().unwrap_or(&PathBuf::from(".")).to_path_buf()
    };

    let manager = migration::MigrationManager::new(&data_dir);

    let result = db.query("SELECT version FROM _migrations ORDER BY version")?;
    let applied: Vec<u32> = match &result {
        QueryResult::Rows(rows) => rows.iter()
            .filter_map(|r| r.get("version").and_then(|v| v.as_i64()).map(|v| v as u32))
            .collect(),
        _ => Vec::new(),
    };

    let statuses = manager.get_status(&applied)?;

    if statuses.is_empty() {
        println!("{}", "No migrations found.".yellow());
        return Ok(());
    }

    println!("\n{}", "Migration Status:".cyan().bold());
    println!("{}", "─".repeat(50).dimmed());
    println!("{:<10} {:<30} {:<10}", "Version", "Name", "Status");
    println!("{}", "─".repeat(50).dimmed());

    for s in &statuses {
        let status = if s.applied {
            "✓ Applied".green().to_string()
        } else {
            "○ Pending".yellow().to_string()
        };
        println!("{:<10} {:<30} {:<10}", s.version, s.name, status);
    }

    println!("{}", "─".repeat(50).dimmed());
    let applied_count = statuses.iter().filter(|s| s.applied).count();
    let pending_count = statuses.iter().filter(|s| !s.applied).count();
    println!("{} applied, {} pending", applied_count.to_string().green(), pending_count.to_string().yellow());

    Ok(())
}

fn run_migrate_rollback(db_path: PathBuf, target_version: u32) -> anyhow::Result<()> {
    let mut db = Database::open(&db_path)?;

    if db.has_password() {
        let password = prompt_password("Password: ")?;
        if !db.authenticate(&password)? {
            eprintln!("{}", "Authentication failed!".red().bold());
            return Ok(());
        }
    }

    let data_dir = if db_path.is_dir() {
        db_path.clone()
    } else {
        db_path.parent().unwrap_or(&PathBuf::from(".")).to_path_buf()
    };

    let manager = migration::MigrationManager::new(&data_dir);

    let result = db.query("SELECT version, name FROM _migrations ORDER BY version DESC")?;
    let applied: Vec<(u32, String)> = match &result {
        QueryResult::Rows(rows) => rows.iter()
            .filter_map(|r| {
                let v = r.get("version").and_then(|v| v.as_i64()).map(|v| v as u32)?;
                let n = r.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                Some((v, n))
            })
            .collect(),
        _ => Vec::new(),
    };

    let mut rolled_back = 0;
    for (version, name) in &applied {
        if *version <= target_version {
            break;
        }

        if let Some(migration) = manager.get_by_version(*version)? {
            if !migration.down.is_empty() {
                println!("{} {}", "Rolling back migration:".yellow(), name);
                db.query(&migration.down)?;
                db.query(&format!("DELETE FROM _migrations WHERE version = {}", version))?;
                println!("  {}", "Done.".green());
                rolled_back += 1;
            }
        }
    }

    if rolled_back == 0 {
        println!("{}", "No migrations to rollback.".yellow());
    } else {
        println!("{} {} migrations rolled back.", "Success:".green(), rolled_back);
    }

    Ok(())
}

fn run_migrate_dry_run(db_path: PathBuf, version: u32) -> anyhow::Result<()> {
    let data_dir = if db_path.is_dir() {
        db_path.clone()
    } else {
        db_path.parent().unwrap_or(&PathBuf::from(".")).to_path_buf()
    };

    let manager = migration::MigrationManager::new(&data_dir);

    match manager.get_by_version(version)? {
        Some(migration) => {
            println!("{}", manager.dry_run(&migration));
        }
        None => {
            eprintln!("{} Migration version {} not found.", "Error:".red().bold(), version);
        }
    }

    Ok(())
}
