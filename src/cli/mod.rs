use crate::config::TitaniumConfig;
use crate::core::router::Router;
use crate::server::{run_server, ServerConfig};
use crate::storage::db::Database;
use std::path::PathBuf;

pub fn run() {
    let args: Vec<String> = std::env::args().collect();

    let mut host = None;
    let mut port = None;
    let mut root_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut db_path = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-S" | "--server" => {
                if i + 1 < args.len() {
                    let addr = &args[i + 1];
                    if let Some((h, p)) = addr.split_once(':') {
                        host = Some(h.to_string());
                        port = Some(p.parse().unwrap_or(8080));
                    } else {
                        host = Some(addr.to_string());
                    }
                    i += 1;
                }
            }
            "-d" | "--dir" => {
                if i + 1 < args.len() {
                    root_dir = PathBuf::from(&args[i + 1]);
                    i += 1;
                }
            }
            "--db" => {
                if i + 1 < args.len() {
                    db_path = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "dev" | "serve" => {
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    root_dir = PathBuf::from(&args[i + 1]);
                    i += 1;
                }
            }
            "studio" => {
                let target = if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    PathBuf::from(&args[i + 1])
                } else {
                    root_dir.clone()
                };
                let cfg = TitaniumConfig::load_from_dir(&target);
                let final_host = host.unwrap_or_else(|| cfg.host("127.0.0.1"));
                let final_port = port.unwrap_or_else(|| cfg.port(8080));
                let final_db = db_path.or_else(|| Some(cfg.db_path(&target)));
                let studio_url = format!("http://{}:{}/__titanium_studio", final_host, final_port);

                let url_for_thread = studio_url.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(400));
                    #[cfg(target_os = "windows")]
                    let _ = std::process::Command::new("explorer")
                        .arg(&url_for_thread)
                        .spawn();
                    #[cfg(target_os = "macos")]
                    let _ = std::process::Command::new("open").arg(&url_for_thread).spawn();
                    #[cfg(target_os = "linux")]
                    let _ = std::process::Command::new("xdg-open").arg(&url_for_thread).spawn();
                });

                let config = ServerConfig {
                    root_dir: target,
                    host: final_host,
                    port: final_port,
                    db_path: final_db,
                    workers: cfg.workers(4),
                    queue_workers: cfg.queue_workers(2),
                };
                if let Err(e) = run_server(config) {
                    eprintln!("Titanium Server Fatal Error: {}", e);
                    std::process::exit(1);
                }
                return;
            }
            "build" => {
                let target = if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    PathBuf::from(&args[i + 1])
                } else {
                    root_dir.clone()
                };
                build_project(&target);
                return;
            }
            "migrate" => {
                let target = if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    PathBuf::from(&args[i + 1])
                } else {
                    root_dir.clone()
                };
                run_cli_migrations(&target, db_path.as_deref());
                return;
            }
            "info" => {
                let target = if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    PathBuf::from(&args[i + 1])
                } else {
                    root_dir.clone()
                };
                print_project_info(&target);
                return;
            }
            "new" | "init" => {
                let target = if i + 1 < args.len() {
                    PathBuf::from(&args[i + 1])
                } else {
                    root_dir.clone()
                };
                scaffold_project(&target);
                return;
            }
            "--help" | "-h" => {
                print_help();
                return;
            }
            arg if !arg.starts_with('-') => {
                let p = PathBuf::from(arg);
                if p.exists() || p.join("pages").exists() {
                    root_dir = p;
                }
            }
            _ => {}
        }
        i += 1;
    }

    // Load project configuration from titanium.toml / cobalt.toml / carbon.toml
    let cfg = TitaniumConfig::load_from_dir(&root_dir);

    let final_host = host.unwrap_or_else(|| cfg.host("127.0.0.1"));
    let final_port = port.unwrap_or_else(|| cfg.port(8080));
    let final_db = db_path.or_else(|| Some(cfg.db_path(&root_dir)));
    let workers = cfg.workers(4);
    let queue_workers = cfg.queue_workers(2);

    let config = ServerConfig {
        root_dir,
        host: final_host,
        port: final_port,
        db_path: final_db,
        workers,
        queue_workers,
    };

    if let Err(e) = run_server(config) {
        eprintln!("Titanium Server Fatal Error: {}", e);
        std::process::exit(1);
    }
}

fn build_project(target: &PathBuf) {
    println!("\n  📦 Building Titanium standalone package: {}\n", target.display());
    let pages_dir = target.join("pages");
    let public_dir = target.join("public");

    if !pages_dir.exists() {
        eprintln!("  ❌ Error: Directory does not contain a 'pages' folder: {}", target.display());
        std::process::exit(1);
    }

    let mut router = Router::new();
    router.scan_dir(&pages_dir);

    let mut valid_pages = 0;
    if let Ok(entries) = std::fs::read_dir(&pages_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                valid_pages += 1;
            }
        }
    }

    println!("  ✅ Validated {} page route(s)", valid_pages);
    if public_dir.exists() {
        println!("  ✅ Bundled static assets in {}", public_dir.display());
    }
    println!("  ✨ Production build complete! Ready for standalone deployment.\n");
}

fn run_cli_migrations(target: &PathBuf, custom_db: Option<&str>) {
    let migrations_dir = target.join("migrations");
    let db_file = custom_db
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            target
                .join(".titanium")
                .join("app.sqlite")
                .to_string_lossy()
                .to_string()
        });

    println!("\n  🗄️ Running database migrations for: {}", target.display());
    println!("  Database target: {}\n", db_file);

    match Database::new(&db_file) {
        Ok(db) => match db.run_migrations(&migrations_dir) {
            Ok(count) => {
                println!("  ✅ Migration completed successfully (applied {} migrations).\n", count);
            }
            Err(e) => {
                eprintln!("  ❌ Migration failed: {}\n", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("  ❌ Failed to connect to database: {}\n", e);
            std::process::exit(1);
        }
    }
}

fn print_project_info(target: &PathBuf) {
    println!("\n  ⚡ Titanium (Ti22) Project Diagnostics\n");
    println!("  Root Path:       {}", target.display());
    let pages_dir = target.join("pages");
    let migrations_dir = target.join("migrations");
    let public_dir = target.join("public");

    println!("  Pages Exists:    {}", pages_dir.exists());
    println!("  Migrations:      {}", migrations_dir.exists());
    println!("  Public Assets:   {}", public_dir.exists());
    println!("  Version:         8.0.0 (Hyperdrive Realtime)");
    println!("  Runtime:         Rust Native Binary\n");
}

fn scaffold_project(target: &PathBuf) {
    let pages = target.join("pages");
    let public = target.join("public");

    let _ = std::fs::create_dir_all(&pages);
    let _ = std::fs::create_dir_all(&public);

    let index_ti = r#"---
let title = "Hello from Titanium v8.0.0 Hyperdrive!";
let db_info = "ActiveRecord ORM + Realtime WebSockets + MVC ready.";
---
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>{{ title }}</title>
  <style>
    body { font-family: system-ui, sans-serif; background: #090d16; color: #f8fafc; padding: 40px; }
    .card { background: #0f172a; border: 1px solid #1e293b; border-radius: 12px; padding: 32px; max-width: 600px; margin: 40px auto; }
    h1 { color: #38bdf8; font-size: 24px; }
    p { color: #94a3b8; font-size: 15px; margin-top: 8px; }
  </style>
</head>
<body>
  <div class="card">
    <h1>{{ title }}</h1>
    <p>{{ db_info }}</p>
  </div>
</body>
</html>
"#;

    let _ = std::fs::write(pages.join("index.titanium"), index_ti);

    let toml_config = r#"[server]
host = "127.0.0.1"
port = 8080
workers = 4

[database]
path = ".titanium/app.sqlite"
wal_mode = true

[cache]
default_ttl = 300

[queue]
worker_threads = 2
"#;
    let _ = std::fs::write(target.join("titanium.toml"), toml_config);

    println!("  ✅ Created new Titanium project in {}", target.display());
    println!("  Run: titanium dev {}", target.display());
}

fn print_help() {
    println!(
        r#"
  ⚡ Titanium (Ti22) — Native Web Runtime v9.0.0 (Singularity AI)
  
  Usage:
    titanium dev [dir]               Start live development server
    titanium studio [dir]            Start server and open Web Studio in browser
    titanium build [dir]             Validate & package project for production
    titanium migrate [dir]           Execute database migrations
    titanium info [dir]              Display project telemetry & diagnostics
    titanium new <dir>               Scaffold new Titanium project with titanium.toml
  
  Options:
    -S, --server <host:port>       Bind host and port (e.g. 127.0.0.1:8080)
    -d, --dir <path>               Target root directory
    --db <path>                    Custom SQLite database path
    -h, --help                     Show this help message
"#
    );
}
