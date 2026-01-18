use std::path::PathBuf;
use clap::{Parser, Subcommand};
use nextup::{questionnaire,List};
use nextup::config::Config;
use nextup::secret::{add_secret,delete_secret};
use nextup::datasource::DataSource;
use anyhow::*;

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

    /// Enable extra verbose and extra capricious trace output
    #[arg(short, long)]
    trace: bool,

    /// If no command is given, print the next task
    #[command(subcommand)]
    subcommand: Option<SubCommand>,
}

impl Args {
    pub fn config(&self) -> Result<Config> {
        let path = Config::filepath_or_default(&self.config);
        let mut cfg = match path {
            Some(resolved) => Config::from_filepath(&resolved).unwrap_or_default(),
            None => Config::default(),
        };
        if self.list.is_some() {
            cfg.list.replace_range(.., self.list.as_ref().ok_or(anyhow!("No list specified"))?);
        }
        Ok(cfg)
    }
    pub async fn data_source(&self) -> Result<DataSource> {
        let cfg = self.config()?;
        let ds = cfg.data_source().await
            .context("Failed to open data source")?;
        Ok(ds)
    }
    pub async fn list(&self) -> Result<(List, DataSource)> {
        let mut ds = self.data_source().await?;
        let list = List::load(&mut ds).await?;
        Ok((list, ds))
    }
}

fn nextup(list: &List) {
    match list.nextup() {
        Some(task) => println!("Next up: {}", task),
        None => println!("All caught up!"),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    use log::LevelFilter;
    colog::default_builder()
        .filter(
            None,
            if args.trace {
                LevelFilter::Trace
            } else if args.debug {
                LevelFilter::Debug
            } else {
                LevelFilter::Info
            },
        ).init();

    match args.subcommand {
        Some(SubCommand::List) => {
            let (list, _) = args.list().await?;
            for (i, task) in list.iter().enumerate() {
                println!("{}: {}", i+1, task);
            }
            nextup(&list);
        },
        Some(SubCommand::ListLists) => {
            let lists = args.data_source().await?.list_lists().await?;
            for list in lists {
                println!("{}", list);
            }
        },
        Some(SubCommand::All) => {
            let tasks = args.data_source().await?.all_first_tasks().await?;
            for (list, task) in tasks {
                println!("{}: {}", list, task);
            }
        },
        Some(SubCommand::Debug {ref subcommand}) => {
            match subcommand {
                DebugCommands::Inspect => {
                    println!("Tasks:");
                    let (list, _) = args.list().await?;
                    for (i, task) in list.iter().enumerate() {
                        println!("{}: {}", i+1, task);
                    }
                    println!("Strings:");
                    let (list, _) = args.list().await?;
                    for (i, range) in list.strings().iter().enumerate() {
                        let free = if range.free { "free" } else { "used" };
                        println!("{}: {} - {} ({}): {}", i, range.range.start, range.range.end, free, range.value);
                    }
                },
                DebugCommands::DBPath => {
                    println!("{:?}", args.config()?.data_source);
                },
                DebugCommands::ConfigPath => {
                    match Config::filepath_or_default(&args.config) {
                        Some(path) => println!("{}", path.display()),
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
                    let mut ds = args.data_source().await?;
                    ds.nuke().await?;
                    let mut list = List::new();
                    list.save(&mut ds).await?;
                },
            }
        },
        Some(SubCommand::Add { ref task }) => {
            let (mut list, mut ds) = args.list().await?;
            let mut cursor = list.add(&task);
            questionnaire(&mut cursor).await?;
            list.save(&mut ds).await?;
            nextup(&list)
        },
        Some(SubCommand::Replace { index, ref task }) => {
            let (mut list, mut ds) = args.list().await?;
            list.replace(index-1, &task)?;
            list.save(&mut ds).await?;
            nextup(&list)
        },
        Some(SubCommand::Complete) => {
            let (mut list, mut ds) = args.list().await?;
            let mut cursor = list.complete()?;
            questionnaire(&mut cursor).await?;
            list.save(&mut ds).await?;
            nextup(&list)
        },
        Some(SubCommand::Delete { index }) => {
            let (mut list, mut ds) = args.list().await?;
            let mut cursor = list.delete(index-1)?;
            questionnaire(&mut cursor).await?;
            list.save(&mut ds).await?;
            nextup(&list)
        },
        Some(SubCommand::Defer) => {
            let (mut list, mut ds) = args.list().await?;
            let mut cursor = list.defer();
            questionnaire(&mut cursor).await?;
            list.save(&mut ds).await?;
            nextup(&list)
        },
        Some(SubCommand::AddSecret { name }) => {
            add_secret(&name).await?;
        },
        Some(SubCommand::DeleteSecret { name }) => {
            delete_secret(&name)?;
        },
        None => {
            let (list, _) = args.list().await?;
            nextup(&list);
        },
    }
    Ok(())
}
