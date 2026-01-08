use std::path::PathBuf;
use clap::{Parser, Subcommand};
use nextup::{questionnaire,List};
use nextup::config::Config;
use nextup::secret::{add_secret,delete_secret};

#[derive(Subcommand, Debug)]
enum DebugCommands {
    /// Inspect the current state of the list and strings
    Inspect,

    /// Nuke the database
    Nuke,

    /// Inspect the data source
    DBPath,

    /// Display the path to the config file file, if one is found,
    /// or suggest locations to create one.
    ConfigPath,
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    /// List all tasks in semi-sorted order
    List,

    /// List all lists in the data source
    ListLists,

    /// Show the next task from all lists
    All,

    /// Add a new task to the list: may ask you some ranking questions
    Add {
        task: String,
    },

    /// Replaces the description of a task by index: use `nextup list` to find the index
    Replace {
        index: usize,
        task: String,
    },

    /// Completes the next task: may ask you some ranking questions
    Complete,

    /// Defer the next task: may ask you some ranking questions
    Defer,

    /// Debug commands
    Debug {
        #[command(subcommand)]
        subcommand: DebugCommands,
    },

    /// Delete a task by index. Use `nextup list` to find the index: may ask you some ranking
    /// questions.
    Delete {
        index: usize,
    },

    AddSecret {
        name: String,
    },
    DeleteSecret {
        name: String,
    },
}

#[derive(Parser, Debug)]
struct Args {
    /// Use a list other than the default specified in the config
    #[arg(short, long)]
    list: Option<String>,

    /// Use a config file other than the default
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Enable my capricious debug output
    #[arg(short='g', long)]
    debug: bool,

    /// If no command is given, print the next task
    #[command(subcommand)]
    subcommand: Option<SubCommand>,
}

impl Args {
    pub fn config(&self) -> Config {
        let path = Config::filepath_or_default(&self.config);
        let mut cfg = match path {
            Some(resolved) => Config::from_filepath(&resolved).unwrap_or_default(),
            None => Config::default(),
        };
        if self.list.is_some() {
            cfg.list.replace_range(.., self.list.as_ref().unwrap());
        }
        cfg
    }
    
}


#[tokio::main]
async fn main() {
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

    let mut suppress = false;
    match args.subcommand {
        Some(SubCommand::List) => {
            for (i, task) in list.iter().enumerate() {
                println!("{}: {}", i+1, task);
            }
        },
        Some(SubCommand::ListLists) => {
            let lists = ds.list_lists().unwrap();
            for list in lists {
                println!("{}", list);
            }
            suppress = true;
        },
        Some(SubCommand::All) => {
            let tasks = ds.all_first_tasks().unwrap();
            for (list, task) in tasks {
                println!("{}: {}", list, task);
            }
            suppress = true;
        },
        Some(SubCommand::Debug {subcommand}) => {
            suppress = true;
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
            }
        },
        Some(SubCommand::Add { task }) => {
            let mut cursor = list.add(&task);
            questionnaire(&mut cursor).await.unwrap();
        },
        Some(SubCommand::Replace { index, task }) => {
            list.replace(index-1, &task).unwrap();
        },
        Some(SubCommand::Complete) => {
            let mut cursor = list.complete().unwrap();
            questionnaire(&mut cursor).await.unwrap();
        },
        Some(SubCommand::Delete { index }) => {
            let mut cursor = list.delete(index-1).unwrap();
            questionnaire(&mut cursor).await.unwrap();
        },
        Some(SubCommand::Defer) => {
            let mut cursor = list.defer();
            questionnaire(&mut cursor).await.unwrap();
        },
        Some(SubCommand::AddSecret { name }) => {
            suppress = true;
            add_secret(&name).await.unwrap();
        },
        Some(SubCommand::DeleteSecret { name }) => {
            suppress = true;
            delete_secret(&name).unwrap();
        },
        None => (),
    }
    if ! suppress {
        match list.nextup() {
            Some(task) => println!("Next up: {}", task),
            None => println!("All caught up!"),
        }
    }
    let saved = list.save(ds.as_mut());
    if let Err(e) = saved {
        eprintln!("Error saving list: {}", e);
    }
}
