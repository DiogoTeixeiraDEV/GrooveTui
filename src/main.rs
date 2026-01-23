use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
// Importante para habilitar os métodos .take_duration() e .amplify()
use rodio::Source;
use std::{
    io,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};

// --- COMUNICAÇÃO ENTRE TUI E ÁUDIO ---

enum AudioCommand {
    Toggle,
    SetBpm(u64),
    Quit,
}

// --- ESTADO DA APLICAÇÃO ---

struct App {
    bpm: u64,
    is_playing: bool,
    genre: String,
    root_note: String,
    audio_tx: Sender<AudioCommand>,
}

impl App {
    fn new(tx: Sender<AudioCommand>) -> Self {
        Self {
            bpm: 120,
            is_playing: false,
            genre: "Blues".to_string(),
            root_note: "E".to_string(),
            audio_tx: tx,
        }
    }
}

// --- ENGINE DE ÁUDIO ---

fn run_audio_thread(rx: Receiver<AudioCommand>) {
    let (_stream, stream_handle) = rodio::OutputStream::try_default().expect("Falha ao abrir saída de áudio");
    let sink = rodio::Sink::try_new(&stream_handle).expect("Falha ao criar o Sink");

    let mut playing = false;
    let mut bpm = 120;
    let mut last_tick = Instant::now();

    loop {
        // Processa comandos da UI
        while let Ok(cmd) = rx.try_recv() {
            match cmd {
                AudioCommand::Toggle => playing = !playing,
                AudioCommand::SetBpm(new_bpm) => bpm = new_bpm,
                AudioCommand::Quit => return,
            }
        }

        if playing {
            let interval = Duration::from_secs_f64(60.0 / bpm as f64);
            if last_tick.elapsed() >= interval {
                // Cria o som do metrônomo (440Hz por 50ms)
                let source = rodio::source::SineWave::new(440.0)
                    .take_duration(Duration::from_millis(50))
                    .amplify(0.2);
                sink.append(source);
                last_tick = Instant::now();
            }
        }

        // Evita consumo excessivo de CPU
        thread::sleep(Duration::from_millis(1));
    }
}

// --- MAIN LOOP ---

fn main() -> Result<()> {
    // Setup do terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Canal de áudio
    let (tx, rx) = mpsc::channel();
    let audio_thread = thread::spawn(move || run_audio_thread(rx));

    let mut app = App::new(tx);

    loop {
        terminal.draw(|f| ui(f, &app))?;

        // Captura de eventos (Input)
        if event::poll(Duration::from_millis(16))? { // ~60 FPS
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => {
                            let _ = app.audio_tx.send(AudioCommand::Quit);
                            break;
                        }
                        KeyCode::Char(' ') => {
                            app.is_playing = !app.is_playing;
                            let _ = app.audio_tx.send(AudioCommand::Toggle);
                        }
                        KeyCode::Up => {
                            app.bpm += 1;
                            let _ = app.audio_tx.send(AudioCommand::SetBpm(app.bpm));
                        }
                        KeyCode::Down => {
                            if app.bpm > 1 {
                                app.bpm -= 1;
                                let _ = app.audio_tx.send(AudioCommand::SetBpm(app.bpm));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Restaura o terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    let _ = audio_thread.join();
    Ok(())
}

// --- INTERFACE VISUAL ---

fn ui(f: &mut Frame, app: &App) {
    let size = f.size();

    // Layout principal
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Cabeçalho
            Constraint::Min(5),    // Main
            Constraint::Length(3), // Footer
        ])
        .split(size);

    // 1. Cabeçalho
    let header = Paragraph::new(" GUITUI: O Companheiro do Guitarrista Rustáceo ")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).style(Style::default().fg(Color::Cyan)));
    f.render_widget(header, chunks[0]);

    // 2. Área Central (Metrônomo e Harmonia)
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Painel Metrônomo
    let play_status = if app.is_playing { "ON" } else { "OFF" };
    let metronome_text = format!(
        "\n  BPM: {}\n  Status: {}\n\n  Setas UP/DOWN ajustam tempo",
        app.bpm, play_status
    );
    let metronome_block = Paragraph::new(metronome_text)
        .block(Block::default().title(" Metrônomo ").borders(Borders::ALL).style(Style::default().fg(if app.is_playing { Color::Green } else { Color::White })));
    f.render_widget(metronome_block, main_layout[0]);

    // Painel Harmonia (Placeholder para o próximo passo)
    let harmony_text = format!(
        "\n  Nota Dominante: {}\n  Gênero: {}\n\n  (Lógica de acordes virá aqui)",
        app.root_note, app.genre
    );
    let harmony_block = Paragraph::new(harmony_text)
        .block(Block::default().title(" Sugestão de Harmonia ").borders(Borders::ALL));
    f.render_widget(harmony_block, main_layout[1]);

    // 3. Rodapé
    let help_text = " [Space] Play/Pause | [Q] Sair | [↑/↓] BPM ";
    let footer = Paragraph::new(help_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(footer, chunks[2]);
}
