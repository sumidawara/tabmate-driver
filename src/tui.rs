use crossterm::{
    event::{Event, EventStream, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;
use std::sync::Arc;
use tokio::sync::watch;

use crate::driver::{ConnectionStatus, DriverState};

pub async fn run(
    enabled_tx: Arc<watch::Sender<bool>>,
    mut state_rx: watch::Receiver<DriverState>,
) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let result = run_inner(&mut terminal, enabled_tx, &mut state_rx).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_inner(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    enabled_tx: Arc<watch::Sender<bool>>,
    state_rx: &mut watch::Receiver<DriverState>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut event_stream = EventStream::new();
    let mut enabled = true;

    terminal.draw(|f| render(f, &state_rx.borrow(), enabled))?;

    loop {
        tokio::select! {
            _ = state_rx.changed() => {
                terminal.draw(|f| render(f, &state_rx.borrow(), enabled))?;
            }
            Some(Ok(event)) = event_stream.next() => {
                match event {
                    Event::Key(key) => match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') => break,
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                        KeyCode::Char(' ') | KeyCode::Enter => {
                            enabled = !enabled;
                            let _ = enabled_tx.send(enabled);
                            terminal.draw(|f| render(f, &state_rx.borrow(), enabled))?;
                        }
                        _ => {}
                    },
                    Event::Resize(_, _) => {
                        terminal.clear()?;
                        terminal.draw(|f| render(f, &state_rx.borrow(), enabled))?;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

const MIN_WIDTH: u16 = 50;
const MIN_HEIGHT: u16 = 14;

fn render(f: &mut ratatui::Frame, state: &DriverState, enabled: bool) {
    let area = f.area();

    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        let msg = format!(
            "画面が小さすぎます\n現在: {}x{}  最小: {}x{}",
            area.width, area.height, MIN_WIDTH, MIN_HEIGHT
        );
        let warning = Paragraph::new(msg)
            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
        f.render_widget(warning, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(4),
            Constraint::Length(1),
        ])
        .split(area);

    let title = Paragraph::new("TABMATE Driver")
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    f.render_widget(title, chunks[0]);

    let (indicator, status_label, color) = match &state.status {
        ConnectionStatus::Disabled   => ("■", "停止中", Color::DarkGray),
        ConnectionStatus::Searching  => ("◌", "探索中...", Color::Yellow),
        ConnectionStatus::Connecting => ("◌", "接続中...", Color::Yellow),
        ConnectionStatus::Connected  => ("●", "接続済み", Color::Green),
    };
    let last_btn = state.last_button.as_deref().unwrap_or("-");
    let status_text = format!("{} {}  |  最後: {}", indicator, status_label, last_btn);
    let status = Paragraph::new(status_text)
        .block(Block::default().title("状態").borders(Borders::ALL))
        .style(Style::default().fg(color));
    f.render_widget(status, chunks[1]);

    let log_height = chunks[2].height.saturating_sub(2) as usize;
    let items: Vec<ListItem> = state.logs.iter()
        .rev()
        .take(log_height)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|l| ListItem::new(Span::raw(l.as_str())))
        .collect();
    let log_list = List::new(items)
        .block(Block::default().title("ログ").borders(Borders::ALL));
    f.render_widget(log_list, chunks[2]);

    let toggle_label = if enabled { "[Space] 停止" } else { "[Space] 開始" };
    let help = Paragraph::new(format!("{}  [q] 終了", toggle_label))
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[3]);
}
