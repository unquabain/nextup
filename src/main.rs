use std::path::PathBuf;
use clap::{Parser, Subcommand};
use nextup::{questionnaire,List};

#[derive(Subcommand, Debug)]
enum SubCommand {
    List,
    Add {
        task: String,
    },
    Complete,
    Defer,
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
        Some(SubCommand::Add { task }) => {
            let mut cursor = list.add(&task);
            questionnaire(&mut cursor).unwrap();
        },
        Some(SubCommand::Complete) => {
            let mut cursor = list.complete();
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
    let saved = list.save(&dbfile);
    if let Err(e) = saved {
        eprintln!("Error saving list: {}", e);
    }
}
