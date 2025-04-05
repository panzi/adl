use std::{
    fs::{create_dir_all, read_dir, File},
    io::Write,
    path::PathBuf,
    time::SystemTime,
};

use httpdate::fmt_http_date;

const README_CONTENTS: &str = include_str!("templates/readme_template.md");
const ADR_CONTENTS: &str = include_str!("templates/adr_template.md");
const HELP_CONTENTS: &str = include_str!("templates/help.txt");

#[inline]
fn rebuild_readme() -> std::io::Result<()> {
    rebuild_readme_with(&get_all_files_in_adr_dir()?)
}

fn rebuild_readme_with(files: &[impl AsRef<str>]) -> std::io::Result<()> {
    let path: PathBuf = [".", "adr", "README.md"].iter().collect();

    let now = SystemTime::now();
    let date = fmt_http_date(now);
    let output = README_CONTENTS.replace("{{timestamp}}", &date);

    let mut formatted_files = Vec::with_capacity(files.len());

    for file in files {
        let file = file.as_ref();
        let new_str = format!(" - [{file}]({file})");
        formatted_files.push(new_str);
    }

    let replacement = formatted_files.join("\n");

    let with_contents = output.replace("{{contents}}", &replacement);

    let mut f = File::create(path)?;
    f.write_all(with_contents.as_bytes())
}

fn generate_adr(n: usize, name: &str) -> std::io::Result<()> {
    let safe_name = name.replace(|ch: char| ch == ' ' || ch == '/' || ch == '\\', "-");
    let file_name: PathBuf = [".", "adr", &format!("{n:05}-{safe_name}.md")].iter().collect();

    let heading = format!("{n:05} - {name}");
    let contents = ADR_CONTENTS.replace("{{name}}", &heading);

    let mut f = File::create(file_name)?;
    f.write_all(contents.as_bytes())
}

fn ensure_dirs_exist() -> std::io::Result<()> {
    let path: PathBuf = [".", "assets"].iter().collect();
    create_dir_all(path)
}

fn get_all_files_in_adr_dir() -> std::io::Result<Vec<String>> {
    let mut file_list = Vec::new();

    for entry in read_dir("adr")? {
        let file_name = entry?.file_name();
        if file_name != "README.md" && file_name != "assets" {
            file_list.push(file_name.to_string_lossy().to_string());
        }
    }

    file_list.sort();

    Ok(file_list)
}

fn main() -> std::io::Result<()> {
    let mut args = std::env::args();

    // ignore the program binary name
    args.next();

    let Some(cmd) = args.next() else {
        println!("{HELP_CONTENTS}");
        return Ok(());
    };

    match cmd.as_str() {
        "create" => {
            ensure_dirs_exist()?;

            let name = args.reduce(|mut acc, arg| {
                acc.push_str(" ");
                acc.push_str(&arg);
                acc
            }).unwrap_or_default();

            if name.is_empty() {
                eprintln!("No name supplied for the ADR. Command should be: adl create Name of ADR here");
                std::process::exit(1);
            }

            let file_list = get_all_files_in_adr_dir()?;
            generate_adr(file_list.len(), &name)?;
            rebuild_readme_with(&file_list)?;
        }
        "regen" => {
            ensure_dirs_exist()?;
            rebuild_readme()?;
        }
        _ => {
            eprintln!("Unknown command: {cmd}");
            println!("\n{HELP_CONTENTS}");
            std::process::exit(1);
        }
    }

    Ok(())
}
