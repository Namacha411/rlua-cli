use std::{fs::File, io::Read, process::ExitCode};

use clap::Parser;
use rlua::{Error, Lua, MultiValue};
use rustyline::Editor;

fn main() -> ExitCode {
    let args = Args::parse();

    if args.version {
        lua_version();
        return ExitCode::SUCCESS;
    }

    if let Some(path) = args.script {
        exec_file(path);
    }

    if let Some(script) = args.execute {
        execute(script);
        return ExitCode::SUCCESS;
    }

    if let Some(script) = args.interactive {
        execute(script);
        repl();
        return ExitCode::SUCCESS;
    }

    repl();
    ExitCode::SUCCESS
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// execute string
    #[clap(short)]
    execute: Option<String>,

    /// enter interactive mode after executing script
    #[clap(short)]
    interactive: Option<String>,

    /// show version information
    #[clap(short)]
    version: bool,

    /// script
    #[clap(short)]
    script: Option<String>,
}

fn lua_version() {
    Lua::new().context(|lua| {
        lua.load("print('Rlua ' .. _VERSION)")
            .exec()
            .expect("error: failed to execute lua");
    });
}

fn execute(lua: String) {
    Lua::new().context(|lua_ctx| {
        lua_ctx
            .load(&lua)
            .exec()
            .expect("error: failed to execute lua");
    });
}

fn exec_file(path: String) {
    let mut file = File::open(path).expect("file not found.");
    let mut source = String::new();
    file.read_to_string(&mut source)
        .expect("something went wrong reading the file.");
    Lua::new().context(|lua_ctx| lua_ctx.load(source.as_bytes()).exec().expect("error"));
}

/// lua repl
///
/// from https://github.com/amethyst/rlua/blob/master/examples/repl.rs
fn repl() {
    lua_version();

    Lua::new().context(|lua| {
        let mut editor = Editor::<()>::new().expect("failed to create editor.");

        loop {
            let mut prompt = "> ";
            let mut line = String::new();

            loop {
                match editor.readline(prompt) {
                    Ok(input) => line.push_str(&input),
                    Err(_) => return,
                }

                match lua.load(&line).eval::<MultiValue>() {
                    Ok(values) => {
                        editor.add_history_entry(line);
                        println!(
                            "{}",
                            values
                                .iter()
                                .map(|value| format!("{:?}", value))
                                .collect::<Vec<_>>()
                                .join("\t")
                        );
                        break;
                    }
                    Err(Error::SyntaxError {
                        incomplete_input: true,
                        ..
                    }) => {
                        // continue reading input and append it to `line`
                        line.push_str("\n"); // separate input lines
                        prompt = ">> ";
                    }
                    Err(e) => {
                        eprintln!("error: {}", e);
                        break;
                    }
                }
            }
        }
    });
}
