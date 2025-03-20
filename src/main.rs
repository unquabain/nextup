use std::path::PathBuf;
use clap::{Parser, Subcommand};
use nextup::{questionnaire,List};
use nextup::config::Config;

#[derive(Subcommand, Debug)]
enum DebugCommands {
    Inspect,
    Nuke,
    DBPath,
    ConfigPath,
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
    #[arg(short, long)]
    config: Option<PathBuf>,

    #[arg(short='g', long)]
    debug: bool,

    #[command(subcommand)]
    subcommand: Option<SubCommand>,
}

impl Args {
    pub fn config(&self) -> Config {
        let path = Config::filepath_or_default(&self.config);
        match path {
            Some(resolved) => Config::from_filepath(&resolved).unwrap_or_default(),
            None => Config::default(),
        }
    }
    
}


fn main() {
    let args = Args::parse();

    use log::LevelFilter;
    colog::default_builder()
        .filter(None, if args.debug { LevelFilter::Debug } else { LevelFilter::Info })
        .init();

    let config = args.config();
    let mut ds = config.data_source().unwrap();
    let mut list = match List::load(ds.as_mut()) {
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
                    println!("{:?}", ds);
                },
                DebugCommands::ConfigPath => {
                    match Config::filepath_or_default(&args.config) {
                        Some(path) => println!("{:?}", path),
                        None => {
                            println!("No config file found");
                            println!("Try creating one in one of the following locations:");
                            println!("- ./.nextup.toml");
                            println!("- ./.nextup.conf");
                            println!("- {}/nextup/config.toml", dirs::config_dir().unwrap_or("./".into()).to_string_lossy());
                            println!("- {}/nextup/config.conf", dirs::config_dir().unwrap_or("./".into()).to_string_lossy());
                            println!("- /etc/nextup.toml");
                        },
                    }
                },
                DebugCommands::Nuke => {
                    ds.nuke().unwrap();
                    list = List::new();
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
    let saved = list.save(ds.as_mut());
    if let Err(e) = saved {
        eprintln!("Error saving list: {}", e);
    }
}
