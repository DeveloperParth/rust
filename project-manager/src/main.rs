use std::{
    env,
    io::{self, Write},
};
const VALID_FLAGS: [&str; 8] = [
    "-t", "--title", "-d", "--desc", "-s", "--status", "-h", "--help",
];
const VALID_STATUS: [&str; 4] = ["idea", "active", "done", "canceled"];
fn main() {
    let args: Vec<String> = env::args().collect();
    let flags: Vec<String> = parsed_flags(&args);
    if args.len() < 2 {
        print_help(true);
        return;
    }
    if flags.contains(&"-h".to_string()) || flags.contains(&"--help".to_string()) {
        print_help(true);
        return;
    }
    let command = get_command(&args);

    if !is_valid_command(&command) {
        println!("Invalid command: {}", command);
        print_help(true);
        std::process::exit(1);
    }
    let con = sqlite::open("projects.db").unwrap();
    migrate(&con);
    print_title();
    match command.as_str() {
        "mk" => create_project(&con, &args),
        "ls" => list_projects(&con),
        "up" => update_project(&con),
        "rm" => remove_project(&con, &args),
        _ => {
            println!("Invalid command: {}", command);
            print_help(true);
            std::process::exit(1);
        }
    }
}

fn create_project(con: &sqlite::Connection, args: &Vec<String>) {
    let mut title = String::new();
    let mut description = String::new();
    let mut status = String::new();

    for (i, arg) in args.iter().enumerate() {
        if arg == "-t" || arg == "--title" {
            title = args[i + 1].clone();
        }
        if arg == "-d" || arg == "--desc" {
            description = args[i + 1].clone();
        }
        if arg == "-s" || arg == "--status" {
            status = args[i + 1].clone();
        }
    }

    if title.is_empty() {
        println!("Please provide a title for the project");
        io::stdin()
            .read_line(&mut title)
            .expect("Failed to read line");
        title = title.trim().to_string();
        if title.is_empty() {
            println!("Invalid title: {}", title);
            println!("Exiting program");
            std::process::exit(1);
        }
    }
    if title.len() > 50 {
        println!("Title is too long. Max allowed size is 50 chars, Consider using description");
        println!("Exiting program");
        std::process::exit(1);
    }
    if status.is_empty() {
        status = "idea".to_string();
    }
    if !VALID_STATUS.contains(&status.as_str()) {
        status = String::new();
        println!("Invalid status: {}", status);
        println!("Valid status: idea, active, done, canceled");
        print!("Please provide a valid status: ");
        _ = io::stdout().flush();
        io::stdin()
            .read_line(&mut status)
            .expect("Failed to read line");
        status = status.trim().to_string();
        if !VALID_STATUS.contains(&status.as_str()) {
            println!("Invalid status: {}", status);
            println!("Exiting program");
            std::process::exit(1);
        }
    }
    con.execute(
        format!(
            "
        INSERT INTO projects (title, description, status)
        VALUES ('{}', '{}', '{}')
        ",
            title, description, status
        )
        .as_str(),
    )
    .unwrap();
    println!("Project created");
}

fn list_projects(con: &sqlite::Connection) {
    let query = "SELECT * FROM projects";
    let mut titles: Vec<String> = vec![];
    let mut descriptions: Vec<String> = vec![];
    let mut statuses: Vec<String> = vec![];
    let mut ids: Vec<i32> = vec![];

    let mut title_max_length = 0;
    let mut description_max_length = 0;
    con.iterate(query, |pairs| {
        // let mut id = 0;
        // let mut title = String::new();
        // let mut description = String::new();
        // let mut status = String::new();
        // for &(column, value) in pairs.iter() {
        //     // println!("{} = {}", column, value.unwrap());
        //     match column {
        //         "id" => id = value.unwrap().parse().unwrap(),
        //         "title" => title = value.unwrap().to_string(),
        //         "description" => description = value.unwrap().to_string(),
        //         "status" => status = value.unwrap().to_string(),
        //         _ => {}
        //     }
        // }
        // println!(
        //     "{0: <1} | {1: <50} | {2: <50} | {3: <10}",
        //     id, title, description, status
        // );
        for &(column, value) in pairs.iter() {
            match column {
                "title" => {
                    if value.unwrap().len() > title_max_length {
                        title_max_length = value.unwrap().len();
                    }
                    titles.push(value.unwrap().to_string())
                }
                "description" => {
                    if value.unwrap().len() > description_max_length {
                        description_max_length = value.unwrap().len();
                    }
                    descriptions.push(value.unwrap().to_string())
                }
                "status" => statuses.push(value.unwrap().to_string()),
                "id" => ids.push(value.unwrap().parse().unwrap()),
                _ => {}
            }
        }
        true
    })
    .unwrap();
    println!(
        "{0: <3} | {1: <title_width$} | {2: <desc_width$} | {3: <10}",
        "ID",
        "Title",
        "Description",
        "Status",
        title_width = title_max_length,
        desc_width = description_max_length
    );
    for (i, title) in titles.iter().enumerate() {
        println!(
            "{0: <3} | {1: <title_width$} | {2: <desc_width$} | {3: <10}",
            ids[i],
            title,
            descriptions[i],
            statuses[i],
            title_width = title_max_length,
            desc_width = description_max_length
        );
    }
    println!("Listing all projects");
}

fn remove_project(con: &sqlite::Connection, args: &Vec<String>) {
    if args.len() < 3 {
        println!("Please provide a project id to remove");
        return;
    }
    let id: i32 = args[2].parse().unwrap();
    let res = con.execute(
        format!(
            "
        DELETE FROM projects
        WHERE id = {}
        ",
            id
        )
        .as_str(),
    );
    match res {
        Ok(_) => {
            println!("Project removed");
        }
        Err(e) => {
            println!("Error removing project: {}", e);
        }
    }
}

fn update_project(_: &sqlite::Connection) {
    println!("Updating a project");
}

fn migrate(con: &sqlite::Connection) {
    con.execute(
        "
        CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT,
            status TEXT
        )
        ",
    )
    .unwrap()
}

fn parsed_flags(args: &Vec<String>) -> Vec<String> {
    let mut flags: Vec<String> = Vec::new();
    for arg in args {
        if arg.starts_with("-") {
            if !VALID_FLAGS.contains(&arg.as_str()) {
                println!("Invalid flag: {}", arg);
                print_help(true);
                std::process::exit(1);
            }
            flags.push(arg.clone());
        }
    }
    flags
}

fn print_help(should_print_title: bool) {
    if should_print_title {
        print_title();
    }
    println!("Usage: project [command] [options]");
    println!();
    println!("Commands:");
    println!("  mk     Create a new project");
    println!("  ls      List all projects");
    println!("  rm      Remove a project");
    println!();
    println!("Options:");
    println!("  -t, --title     Title of the project");
    println!("  -d, --desc      Description of the project");
    println!("  -s  --status    Status of the project");
    println!("  -h, --help      Print this help message");
}

fn print_title() {
    let s = r"
  _____           _           _   
 |  __ \         (_)         | |  
 | |__) | __ ___  _  ___  ___| |_ 
 |  ___/ '__/ _ \| |/ _ \/ __| __|
 | |   | | | (_) | |  __/ (__| |_ 
 |_|   |_|  \___/| |\___|\___|\__|
                _/ |              
               |__/               
    ";
    println!("{}", s);
}

fn get_command(args: &Vec<String>) -> String {
    let mut command = String::new();
    if args.len() < 2 {
        return command;
    }
    command = args[1].clone();
    command
}

fn is_valid_command(command: &String) -> bool {
    let valid_commands = ["mk", "ls", "rm"];
    valid_commands.contains(&command.as_str())
}
