# Nextup

Nextup is a small command-line tool to manage your task lists in priority order. It uses the well-known heap sort/priority queue algorithm to keep your to-do list in a semi-ordered state so that the most urgent task is always at the top of the list.

## Motivation

Just learning and practicing Rust, which I thought would be like re-learning C, but very much is not.

I used to write some C, then I wrote a lot of PHP, then a lot of Python, a tiny bit of Java and C# here and there. Then I spent years writing Ruby, and now I mostly write Go. It seemed like Rust would be both a logical next step, and a kind of full circle.

This is very similar to the web version you can find [on my website](https://www.unquabain.com/decider-rs).

## Options

Most importantly, the `--help` option shows you all the options and subcommands that are available.

The `--debug` option turns on my rather capricious debug logging.

### Configuration

You can specify the configuration file with `--config`. Or, if you pass it the `debug config-path` subcommand,
it will show you all the default places where it looks for a config file.

The config file might look like this:

```toml
data_source = "postgres"
list = "default"

[bincode]
path = "./"

[postgres]
connection_string = "postgres://benforsberg@localhost:5432/benforsberg"
```

The `data_source` field can be set to `bincode` or `postgres`.

If it's set to `bincode`, the `bincode` section will be used to configure the path to the database file.
The `list` parameter will become a file at the location specified by `path`.

If it's set to `postgres`, the `postgres` section will be used to configure the connection string to the database. 

## Basic commands

### Help

The most important command is `help` that can be used to list all the other options, commands, and subcommands.

### Next Up

Given no arguments, the command will print out your most pressing task, or the text "All caught up!"

### Add

```
nextup add "Fire Jeffrey; he's awful."
```

When you add a new task, you will be asked to rank a small number of tasks in order to find a place for the new task. There should be about $log(n)$ questions in the questionnaire.

When you're done, it will print the (maybe new) top item.

### Complete

```
nextup complete
```

This tells `nextup` that you've done the top item on the list. As when you add a new item, you'll be asked $log(n)$ questions ranking other tasks in your queue. The next top priority is printed, or `All caught up!` if you've completed
the last one.

### Defer

Using heap sort to manage your priority list is all very crystal palace. In the real world, priorities change. Sometimes you can't do that one thing now. If (when) this happens, you can always

```
nextup defer
```

to push the task down the list. You'll be asked the usual $log(n)$ questions to figure out what your next task should be instead.

### List

Surprised that your next task isn't what you thought it would be? Use the `list` subcommand to see what's waiting for you in your queue.

```
nextup list
```

The list won't be exactly sorted. Your top priority will be ranked #1. Your next two priorities will be numbers #2 and #3, but not necessarily in any particular order. Next, your four, third-tier tasks will appear jumbled up as items #4 through #8, and so on.

If there's something in the list that is no longer relevant, you can always:

### Delete

Using the number from `nextup list` you can get rid of one task from the middle of your list.

```
nextup delete 5
```

Nextup will need to ask you about a few of your remaining tasks to figure out how to fill the gap.

### Debug

The debug command groups together some low-level maintenance tools.

#### Inspect

```
nextup debug inspect
```

This prints out the contents of the data file and the strings table.

#### Nuke

```
nextup debug nuke
```

This deletes the data file, in case things have gone irreparably bad somehow.

#### DB Path

```
nextup debug db-path
```

Where does Nextup store this list? This command shows you.

## Building

It's a Rust project. I dunno, just do `cargo build` like I did.
