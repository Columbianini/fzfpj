use regex::Regex;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, ClearType},
    style::{Color, SetForegroundColor},
};
use std::io::{stdout, Write};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Project {
    name: String,
    rootPath: String,
    tags: Vec<String>,
    enabled: bool,
}

fn expand_path(path: &str) -> String {
    let home = env::var("USERPROFILE").unwrap_or_else(|_| String::from(""));
    path.replace("$home", &home)
        .replace("~", &home)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json_content = fs::read_to_string(r"C:\Users\Lenovo\Desktop\Rust\RustProject\fzfpj\src\project.json")?;
    let projects: Vec<Project> = serde_json::from_str(&json_content)?;
    let enabled_projects: Vec<Project> = projects.into_iter()
        .filter(|p| p.enabled)
        .collect();

    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, terminal::EnterAlternateScreen)?;

    let mut current_input = String::new();
    let mut selected_index = 0;
    let mut matched_projects: Vec<&Project> = Vec::new();
    let mut page = 0usize;
    let items_per_page = 2usize;

    loop {
        execute!(
            stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )?;

        println!("Search: {}", current_input);
        println!("----------------------------------------");

        // Create regex pattern from input, escape special characters
        let regex = Regex::new(&format!("(?i){}", current_input)).unwrap_or_else(|_| Regex::new("").unwrap());

        // Update matches using regex
        matched_projects = enabled_projects
            .iter()
            .filter(|project| regex.is_match(&project.name))
            .collect();

        // Calculate Pagination
        let total_pages = (matched_projects.len() + items_per_page - 1) / items_per_page;
        page = page.min(total_pages.saturating_sub(1));
        let start_index = page * items_per_page;
        let end_index = start_index + items_per_page;

        // Display matches
        for (i, project) in matched_projects.iter().enumerate().take(end_index).skip(start_index) {
            if i == selected_index {
                execute!(
                    stdout,
                    SetForegroundColor(Color::Green)
                )?;
                println!("> {}", project.name);
                execute!(
                    stdout,
                    SetForegroundColor(Color::Reset)
                )?;
            } else {
                println!("  {}", project.name);
            }
        }

        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Enter => {
                    if !matched_projects.is_empty() {
                        let selected_project = matched_projects[selected_index];
                        let expanded_path = expand_path(&selected_project.rootPath);
                        
                        execute!(stdout, terminal::LeaveAlternateScreen)?;
                        terminal::disable_raw_mode()?;

                        println!("{}", expanded_path);
                        return Ok(());
                    }
                }
                KeyCode::PageUp => {
                    if page > 0 {
                        page -= 1;
                    }
                    selected_index = page * items_per_page;
                }
                KeyCode::PageDown => {
                    if page < total_pages - 1 {
                        page += 1;
                    }
                    selected_index = page * items_per_page;
                }
                KeyCode::Esc => {
                    break;
                }
                KeyCode::Up => {
                    if selected_index > 0 {
                        selected_index -= 1;
                    }
                }
                KeyCode::Down => {
                    if selected_index + 1 < matched_projects.len().min(end_index) {
                        selected_index += 1;
                    }
                }
                KeyCode::Backspace => {
                    current_input.pop();
                    selected_index = 0;
                }
                KeyCode::Char(c) => {
                    current_input.push(c);
                    selected_index = 0;
                }
                _ => {}
            }
        }
    }

    execute!(stdout, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}