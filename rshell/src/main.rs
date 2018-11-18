use std::io::stdin;
use std::io::stdout;
use std::io::Write;
use std::path::Path;
use std::env;
use std::process::Command;
use std::process::Child;
use std::process::Stdio;
use std::str::Split;

use nix::unistd::getcwd;

fn main(){
    loop {

        print!("\n{} >> ", modified_cwd(3));
        stdout().flush().unwrap();
        // create new string to hold the input
        let mut input = String::new();
        // read the next line. hangs until enter is pressed
        stdin().read_line(&mut input).unwrap();

        // must be peekable so we know when we are on the last command
        let mut commands = input.trim().split(" | ").peekable();
        if input.trim().len() > 0 {
            let mut previous_command: Option<Child> = None;
            while let Some(command) = commands.next() {
                let mut parts = command.trim().split_whitespace();
                let command = parts.next().unwrap();
                let args = parts;

                match command {
                    "cd" => {
                        let new_dir = args.peekable().peek().map_or("/", |x| *x);
                        let root = Path::new(new_dir);
                        if let Err(e) = env::set_current_dir(&root) {
                            eprintln!("{}", e);
                        }
                        previous_command = None;
                    },
                    "exit" => return,
                    command => {
                        let stdin = previous_command
                            .map_or(
                                Stdio::inherit(),
                                |output: Child| Stdio::from(output.stdout.unwrap())
                            );
                        let stdout = if commands.peek().is_some() {
                            // there is another command piped behind this one
                            // prepare to send output to the next command
                            Stdio::piped()
                        } else {
                            // there are no more commands piped behind this one
                            // send output to shell stdout
                            Stdio::inherit()
                        };
                        let output = Command::new(command)
                            .args(args)
                            .stdin(stdin)
                            .stdout(stdout)
                            .spawn();
                        // wait on child to exit
                        match output {
                            Ok(output) => { previous_command = Some(output); },
                            Err(e) => {
                                previous_command = None;
                                eprintln!("Command not found: {}", e);
                            }
                        };
                    }
                }
            }


            if let Some(mut final_command) = previous_command {
                // block until the final command is finished
                final_command.wait().unwrap();
            }
        }
    }
}

fn modified_cwd(mut num_folders: u8) -> String {
    // returns a string representing the current
    // working directory, displaying folders going
    // back up to num_folders
    num_folders = num_folders - 1;
    let cwd = getcwd().unwrap();
    let tokens = cwd.to_str().unwrap().clone();
    let mut tokens = tokens.split("/");
    let mut vec_toks: Vec<&str> = Vec::new();
    while let Some(next_cmd) = tokens.next() {
        vec_toks.push(next_cmd);
    }
    if vec_toks.len() > num_folders as usize {
        let mut return_str: String = "/".to_owned();
        vec_toks = vec_toks[(vec_toks.len() - num_folders as usize - 1)..].to_vec();
        for i in 0..vec_toks.len() {
            return_str.push_str(vec_toks[i]);
            return_str.push_str("/");
        }
        return_str
    } else {
        cwd.to_str().unwrap().to_string()
    }
}
