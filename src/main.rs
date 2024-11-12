use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::process::Command;
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
    // Read and parse JSON
    let json_content = fs::read_to_string(r"C:\Users\Lenovo\Desktop\Rust\RustProject\fzfpj\src\project.json")?;
    let projects: Vec<Project> = serde_json::from_str(&json_content)?;
    let enabled_projects: Vec<Project> = projects.into_iter()
        .filter(|p| p.enabled)
        .collect();

    // Set up terminal
    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, terminal::EnterAlternateScreen)?;

    let matcher = SkimMatcherV2::default();
    let mut current_input = String::new();
    let mut selected_index = 0;
    let mut matched_projects: Vec<(i64, &Project)> = Vec::new();

    loop {
        // Clear screen
        execute!(
            stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )?;

        // Show current input
        println!("Search: {}", current_input);
        println!("----------------------------------------");

        // Update matches
        matched_projects = enabled_projects
            .iter()
            .filter_map(|project| {
                matcher
                    .fuzzy_match(&project.name, &current_input)
                    .map(|score| (score, project))
            })
            .collect();
        matched_projects.sort_by_key(|(score, _)| -score);

        // Display matches
        for (i, (_score, project)) in matched_projects.iter().enumerate() {
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

        // Handle input
        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Enter => {
                    if !matched_projects.is_empty() {
                        let selected_project = matched_projects[selected_index].1;
                        let expanded_path = expand_path(&selected_project.rootPath);
                        
                        // Clean up terminal
                        execute!(stdout, terminal::LeaveAlternateScreen)?;
                        terminal::disable_raw_mode()?;

                        // Open explorer
                        Command::new("explorer")
                            .arg(&expanded_path)
                            .spawn()?;

                        return Ok(());
                    }
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
                    if selected_index < matched_projects.len().saturating_sub(1) {
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

    // Clean up
    execute!(stdout, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}