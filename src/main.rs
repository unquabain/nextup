use std::path::PathBuf;
use clap::{Parser, Subcommand};
use nextup::{questionnaire,List};

#[derive(Subcommand, Debug)]
enum DebugCommands {
    Inspect,
    Nuke,
    DBPath,
    Repack,
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    List,
    Add {
        task: String,
    },
    Complete,
    Defer,
    Debug {
        #[command(subcommand)]
        subcommand: DebugCommands,
    },
    Delete {
        index: usize,
    },
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, value_name="DIR")]
    list: Option<String>,

    #[arg(short, long)]
    db: Option<PathBuf>,

    #[arg(short='g', long)]
    debug: bool,

    #[command(subcommand)]
    subcommand: Option<SubCommand>,
}


fn main() {
    let args = Args::parse();

    use log::LevelFilter;
    colog::default_builder()
        .filter(None, if args.debug { LevelFilter::Debug } else { LevelFilter::Info })
        .init();

    let dbfile = 
        match args.db {
            Some(path) => path,
            None => {
                let mut path = dirs::config_dir().unwrap();
                path.push("nextup");
                match args.list {
                    Some(list) => path.push(list),
                    None => path.push("default"),
                }
                path.push("list");
                path
            }
        };

    let mut list = match List::load(&dbfile) {
        Ok(list) => list,
        Err(_) => List::new(),
    };

    match args.subcommand {
        Some(SubCommand::List) => {
            for (i, task) in list.iter().enumerate() {
                println!("{}: {}", i+1, task);
            }
        },
        Some(SubCommand::Debug {subcommand}) =>
            match subcommand {
                DebugCommands::Inspect => {
                    println!("Tasks:");
                    for (i, task) in list.iter().enumerate() {
                        println!("{}: {}", i+1, task);
                    }
                    println!("Strings:");
                    for (i, range) in list.strings().iter().enumerate() {
                        let free = if range.free { "free" } else { "used" };
                        println!("{}: {} - {} ({}): {}", i, range.range.start, range.range.end, free, range.value);
                    }
                },
                DebugCommands::DBPath => {
                    println!("{}", dbfile.display());
                },
                DebugCommands::Nuke => {
                    std::fs::remove_file(&dbfile).unwrap();
                    list = List::new();
                },
                DebugCommands::Repack => {
                    list.repack();
                },
            },
        Some(SubCommand::Add { task }) => {
            let mut cursor = list.add(&task);
            questionnaire(&mut cursor).unwrap();
        },
        Some(SubCommand::Complete) => {
            let mut cursor = list.complete().unwrap();
            questionnaire(&mut cursor).unwrap();
        },
        Some(SubCommand::Delete { index }) => {
            let mut cursor = list.delete(index-1).unwrap();
            questionnaire(&mut cursor).unwrap();
        },
        Some(SubCommand::Defer) => {
            let mut cursor = list.defer();
            questionnaire(&mut cursor).unwrap();
        },
        None => (),
    }
    match list.nextup() {
        Some(task) => println!("Next up: {}", task),
        None => println!("All caught up!"),
    }
    if list.should_repack() {
        list.repack();
    }
    let saved = list.save(&dbfile);
    if let Err(e) = saved {
        eprintln!("Error saving list: {}", e);
    }
}
