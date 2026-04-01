//! TUI module - Terminal UI for real-time cognitive state display

use cognipace_core::types::{FeatureVector, CognitiveState, AttentionState};
use cognipace_adaptations::AdaptationEngine;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Gauge, Paragraph, Widget},
    Terminal,
};
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::sync::mpsc::Receiver;

pub async fn run_tui(
    rx: &mut Receiver<FeatureVector>,
    engine: &mut AdaptationEngine,
    running: std::sync::Arc<AtomicBool>,
) -> anyhow::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // State
    let mut current_state: Option<CognitiveState> = None;
    let mut features_received = 0;

    // Main loop
    while running.load(Ordering::SeqCst) {
        // Check for events
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if let KeyEvent {
                    code: KeyCode::Char('q'),
                    ..
                } = key
                {
                    running.store(false, Ordering::SeqCst);
                }
            }
        }

        // Process incoming features
        while let Ok(features) = rx.try_recv() {
            current_state = Some(engine.process(&features));
            features_received += 1;
        }

        // Draw UI
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Length(5),
                        Constraint::Length(3),
                        Constraint::Min(0),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            // Title
            let title = Paragraph::new("CogniPace - Cognitive Load Monitor")
                .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(title, chunks[0]);

            // Current State
            if let Some(ref state) = current_state {
                let state_text = format!(
                    "{} | Load: {:.0}% | State: {}",
                    state.attention_state.emoji(),
                    state.load_score * 100.0,
                    state.attention_state.description()
                );
                let state_para = Paragraph::new(state_text)
                    .style(Style::default().fg(get_state_color(&state.attention_state)))
                    .block(Block::default().borders(Borders::ALL).title("Current State"));
                f.render_widget(state_para, chunks[1]);

                // Load Gauge
                let gauge = Gauge::default()
                    .block(Block::default().borders(Borders::ALL).title("Cognitive Load"))
                    .gauge_style(Style::default().fg(get_load_color(state.load_score)))
                    .ratio(state.load_score);
                f.render_widget(gauge, chunks[2]);

                // Recommended Action
                let action = Paragraph::new(state.attention_state.recommended_action())
                    .style(Style::default().fg(Color::Yellow))
                    .block(Block::default().borders(Borders::ALL).title("Recommended Action"));
                f.render_widget(action, chunks[3]);

                // Stats
                let stats = format!(
                    "Features processed: {} | Queued notifications: {}",
                    features_received,
                    engine.notification_router().queued_count()
                );
                let stats_para = Paragraph::new(stats)
                    .style(Style::default().fg(Color::Gray))
                    .block(Block::default().borders(Borders::ALL).title("Stats"));
                f.render_widget(stats_para, chunks[4]);
            } else {
                let waiting = Paragraph::new("Waiting for sensor data...")
                    .style(Style::default().fg(Color::Gray))
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(waiting, chunks[1]);
            }
        })?;
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn get_state_color(state: &AttentionState) -> Color {
    match state {
        AttentionState::DeepFocus => Color::Green,
        AttentionState::FlowBuilding => Color::Yellow,
        AttentionState::Fragmented => Color::DarkYellow,
        AttentionState::Fatigued => Color::Red,
        AttentionState::Overloaded => Color::DarkGray,
        AttentionState::Recovering => Color::Blue,
    }
}

fn get_load_color(load: f64) -> Color {
    if load < 0.3 {
        Color::Green
    } else if load < 0.5 {
        Color::Yellow
    } else if load < 0.7 {
        Color::DarkYellow
    } else if load < 0.9 {
        Color::Red
    } else {
        Color::DarkGray
    }
}
