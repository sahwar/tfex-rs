use std::io;
use std::path::PathBuf;
use std::thread;

use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Paragraph, Text, Widget};
use tui::Frame;

use crate::app::App;
use crate::file_ops;

pub fn draw(app: &mut App) -> Result<(), io::Error> {
    let command_string = app.get_command_buffer_as_string();
    let mut reset_error = false;

    let App {
        current_directory,
        terminal,
        directory_contents,
        selection_index,
        error,
        ..
    } = app;

    terminal.hide_cursor()?;

    terminal.draw(|mut f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)].as_ref())
            .split(f.size());

        draw_file_list(
            &mut f,
            chunks[0],
            directory_contents,
            selection_index,
            current_directory,
        );

        //Error & command box drawing
        if let Some(err) = error {
            draw_error(&mut f, chunks[1], err);
            reset_error = true;
        } else {
            draw_command_buffer(&mut f, chunks[1], command_string);
        }
    })?;

    if reset_error {
        thread::sleep(std::time::Duration::from_secs(1));
        app.error = None;
    }

    Ok(())
}

pub fn draw_file_list<B: Backend>(
    frame: &mut Frame<B>,
    area: Rect,
    files: &Vec<file_ops::DirectoryItem>,
    selected_file: &Option<usize>,
    current_dir: &PathBuf,
) {
    let mut names: Vec<Text> = Vec::new();
    let mut sizes: Vec<Text> = Vec::new();
    let inner_rect = Rect::new(area.x + 1, area.y + 1, area.width - 1, area.height - 1); //Shrinking the area by 1 in every direction for the text columns, as border is drawn separately

    //Draw the border
    Block::default()
        .borders(Borders::ALL)
        .title(format!("Contents─{}", current_dir.to_str().unwrap()).as_ref())
        .render(frame, area);

    if files.len() != 0 {
        //Convert DirectoryItems to Text
        for file in files {
            match file {
                file_ops::DirectoryItem::File((path, size)) => {
                    let split: Vec<&str> = path.split('/').collect();
                    let string = String::from(format!("📄 {}\n", split[split.len() - 1 as usize]));
                    names.push(Text::raw(string));
                    sizes.push(Text::raw(format!("{}KB\n", size.to_string())));
                }
                file_ops::DirectoryItem::Directory(path) => {
                    let split: Vec<&str> = path.split('/').collect();
                    let string = String::from(format!("📁 {}\n", split[split.len() - 1 as usize]));
                    names.push(Text::raw(string));
                    sizes.push(Text::raw("\n"));
                }
            }
        }

        //Highlight selected file
        if let Some(selection_index) = selected_file {
            //Get name of selected file
            let selected = match &mut names[*selection_index] {
                Text::Raw(value) => value,
                _ => "",
            }
            .to_string();

            //Replace name of selected file with bold name
            names.insert(
                *selection_index,
                Text::styled(
                    selected,
                    Style::default()
                        .modifier(Modifier::BOLD)
                        .fg(Color::Indexed(2)),
                ),
            );
            names.remove(selection_index + 1);
        }

        //Figure out number of columns and their spacing
        let columns: u16 = (names.len() as f32 / (area.height - 2) as f32).ceil() as u16;
        let column_size: u16 = 100 / columns;
        let mut constraints: Vec<Constraint> = Vec::new();

        //Create the constraints
        for _ in 1..=columns as u32 {
            constraints.push(Constraint::Percentage(column_size));
        }

        //Create the chunks
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(inner_rect);

        for i in 0..=columns - 1 {
            let height: usize = (area.height - 2) as usize; // -2 to account for the border
            let from: usize = (i as usize * height) as usize;
            let mut to: usize = (i as usize * height) + (height);

            if to >= names.len() {
                to = names.len();
            }

            let names_iter = names[from..to].iter();
            let sizes_iter = sizes[from..to].iter();

            Paragraph::new(names_iter)
                .wrap(false)
                .render(frame, chunks[i as usize]);

            Paragraph::new(sizes_iter)
                .alignment(Alignment::Right)
                .wrap(false)
                .render(
                    frame,
                    Rect {
                        //create new Rect that doesn't overlap the border
                        height: chunks[i as usize].height,
                        width: chunks[i as usize].width - 2,
                        x: chunks[i as usize].x,
                        y: chunks[i as usize].y,
                    },
                );
        }
    }
}

pub fn draw_command_buffer<B: Backend>(frame: &mut Frame<B>, area: Rect, command_string: String) {
    let text: Vec<Text> = vec![Text::raw(command_string)];

    Paragraph::new(text.iter())
        .block(Block::default().title("Command").borders(Borders::ALL))
        .render(frame, area);
}

pub fn draw_error<B: Backend>(frame: &mut Frame<B>, area: Rect, error: &String) {
    let text: Vec<Text> = vec![Text::styled(
        error.to_string(),
        Style::default().fg(Color::Red),
    )];

    Paragraph::new(text.iter())
        .block(
            Block::default()
                .title("Error")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Red)),
        )
        .render(frame, area);
}
