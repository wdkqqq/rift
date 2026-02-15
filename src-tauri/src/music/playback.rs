use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use serde::Serialize;
use std::fs::File;
use std::io::BufReader;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug, Serialize, Clone)]
pub struct PlaybackState {
    pub is_loaded: bool,
    pub is_playing: bool,
    pub current_time: f64,
    pub duration: f64,
    pub volume: f32,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            is_loaded: false,
            is_playing: false,
            current_time: 0.0,
            duration: 0.0,
            volume: 0.7,
        }
    }
}

pub struct PlaybackService {
    tx: mpsc::Sender<PlaybackCommand>,
    snapshot: Arc<Mutex<PlaybackState>>,
}

enum PlaybackCommand {
    LoadAndPlay {
        path: String,
        reply: mpsc::Sender<Result<PlaybackState, String>>,
    },
    Play {
        reply: mpsc::Sender<Result<PlaybackState, String>>,
    },
    Pause {
        reply: mpsc::Sender<Result<PlaybackState, String>>,
    },
    Seek {
        position_seconds: f64,
        reply: mpsc::Sender<Result<PlaybackState, String>>,
    },
    SetVolume {
        volume: f32,
        reply: mpsc::Sender<Result<PlaybackState, String>>,
    },
    GetState {
        reply: mpsc::Sender<Result<PlaybackState, String>>,
    },
}

impl PlaybackService {
    pub fn start() -> Self {
        let (tx, rx) = mpsc::channel::<PlaybackCommand>();
        let snapshot = Arc::new(Mutex::new(PlaybackState::default()));
        let snapshot_for_thread = Arc::clone(&snapshot);

        thread::spawn(move || {
            let mut controller = match PlaybackController::new() {
                Ok(controller) => controller,
                Err(error) => {
                    while let Ok(command) = rx.recv() {
                        let _ = command.reply().send(Err(error.clone()));
                    }
                    return;
                }
            };

            while let Ok(command) = rx.recv() {
                let reply = command.reply().clone();
                let result = match command {
                    PlaybackCommand::LoadAndPlay { path, .. } => controller.load_and_play(&path),
                    PlaybackCommand::Play { .. } => controller.play(),
                    PlaybackCommand::Pause { .. } => Ok(controller.pause()),
                    PlaybackCommand::Seek {
                        position_seconds, ..
                    } => controller.seek(position_seconds),
                    PlaybackCommand::SetVolume { volume, .. } => Ok(controller.set_volume(volume)),
                    PlaybackCommand::GetState { .. } => Ok(controller.state()),
                };

                if let Ok(state) = &result {
                    if let Ok(mut snapshot_state) = snapshot_for_thread.lock() {
                        *snapshot_state = state.clone();
                    }
                }

                let _ = reply.send(result);
            }
        });

        Self { tx, snapshot }
    }

    pub fn load_and_play(&self, path: String) -> Result<PlaybackState, String> {
        let (reply_tx, reply_rx) = mpsc::channel::<Result<PlaybackState, String>>();
        self.tx
            .send(PlaybackCommand::LoadAndPlay {
                path,
                reply: reply_tx,
            })
            .map_err(|_| "Playback service is unavailable".to_string())?;
        reply_rx
            .recv()
            .map_err(|_| "Playback service did not respond".to_string())?
    }

    pub fn play(&self) -> Result<PlaybackState, String> {
        let (reply_tx, reply_rx) = mpsc::channel::<Result<PlaybackState, String>>();
        self.tx
            .send(PlaybackCommand::Play { reply: reply_tx })
            .map_err(|_| "Playback service is unavailable".to_string())?;
        reply_rx
            .recv()
            .map_err(|_| "Playback service did not respond".to_string())?
    }

    pub fn pause(&self) -> Result<PlaybackState, String> {
        let (reply_tx, reply_rx) = mpsc::channel::<Result<PlaybackState, String>>();
        self.tx
            .send(PlaybackCommand::Pause { reply: reply_tx })
            .map_err(|_| "Playback service is unavailable".to_string())?;
        reply_rx
            .recv()
            .map_err(|_| "Playback service did not respond".to_string())?
    }

    pub fn seek(&self, position_seconds: f64) -> Result<PlaybackState, String> {
        let (reply_tx, reply_rx) = mpsc::channel::<Result<PlaybackState, String>>();
        self.tx
            .send(PlaybackCommand::Seek {
                position_seconds,
                reply: reply_tx,
            })
            .map_err(|_| "Playback service is unavailable".to_string())?;
        reply_rx
            .recv()
            .map_err(|_| "Playback service did not respond".to_string())?
    }

    pub fn set_volume(&self, volume: f32) -> Result<PlaybackState, String> {
        let (reply_tx, reply_rx) = mpsc::channel::<Result<PlaybackState, String>>();
        self.tx
            .send(PlaybackCommand::SetVolume {
                volume,
                reply: reply_tx,
            })
            .map_err(|_| "Playback service is unavailable".to_string())?;
        reply_rx
            .recv()
            .map_err(|_| "Playback service did not respond".to_string())?
    }

    pub fn get_state(&self) -> Result<PlaybackState, String> {
        let (reply_tx, reply_rx) = mpsc::channel::<Result<PlaybackState, String>>();
        self.tx
            .send(PlaybackCommand::GetState { reply: reply_tx })
            .map_err(|_| "Playback service is unavailable".to_string())?;
        reply_rx
            .recv()
            .map_err(|_| "Playback service did not respond".to_string())?
            .or_else(|_| {
                self.snapshot
                    .lock()
                    .map(|state| state.clone())
                    .map_err(|_| "Playback state mutex is poisoned".to_string())
            })
    }
}

impl PlaybackCommand {
    fn reply(&self) -> &mpsc::Sender<Result<PlaybackState, String>> {
        match self {
            PlaybackCommand::LoadAndPlay { reply, .. } => reply,
            PlaybackCommand::Play { reply } => reply,
            PlaybackCommand::Pause { reply } => reply,
            PlaybackCommand::Seek { reply, .. } => reply,
            PlaybackCommand::SetVolume { reply, .. } => reply,
            PlaybackCommand::GetState { reply } => reply,
        }
    }
}

struct PlaybackController {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    sink: Sink,
    path: Option<String>,
    duration: f64,
    position_offset: f64,
    paused: bool,
    volume: f32,
}

impl PlaybackController {
    fn new() -> Result<Self, String> {
        let (stream, handle) = OutputStream::try_default()
            .map_err(|error| format!("Cannot initialize audio output: {error}"))?;
        let sink =
            Sink::try_new(&handle).map_err(|error| format!("Cannot create audio sink: {error}"))?;

        Ok(Self {
            _stream: stream,
            handle,
            sink,
            path: None,
            duration: 0.0,
            position_offset: 0.0,
            paused: true,
            volume: 0.7,
        })
    }

    fn load_and_play(&mut self, path: &str) -> Result<PlaybackState, String> {
        self.path = Some(path.to_string());
        self.rebuild_sink(0.0, true)?;
        Ok(self.state())
    }

    fn pause(&mut self) -> PlaybackState {
        self.paused = true;
        self.sink.pause();
        self.state()
    }

    fn play(&mut self) -> Result<PlaybackState, String> {
        if self.path.is_none() {
            return Ok(self.state());
        }

        self.paused = false;
        self.sink.play();
        Ok(self.state())
    }

    fn seek(&mut self, position_seconds: f64) -> Result<PlaybackState, String> {
        if self.path.is_none() {
            return Ok(self.state());
        }

        let clamped = if self.duration > 0.0 {
            position_seconds.max(0.0).min(self.duration)
        } else {
            position_seconds.max(0.0)
        };
        let should_play = !self.paused;
        let target = Duration::from_secs_f64(clamped);

        // Fast path is safe only when source starts from 0.
        if self.position_offset == 0.0 && self.sink.try_seek(target).is_ok() {
            let reported = self.sink.get_pos().as_secs_f64();
            if (reported - clamped).abs() <= 1.0 {
                return Ok(self.state());
            }
        }

        self.rebuild_sink(clamped, should_play)?;
        Ok(self.state())
    }

    fn set_volume(&mut self, volume: f32) -> PlaybackState {
        let clamped = volume.clamp(0.0, 1.0);
        self.volume = clamped;
        self.sink.set_volume(clamped);
        self.state()
    }

    fn state(&mut self) -> PlaybackState {
        if !self.paused
            && self.sink.empty()
            && self.duration > 0.0
            && self.position() >= self.duration
        {
            self.paused = true;
        }

        PlaybackState {
            is_loaded: self.path.is_some(),
            is_playing: self.path.is_some() && !self.paused,
            current_time: self.position(),
            duration: self.duration,
            volume: self.volume,
        }
    }

    fn rebuild_sink(&mut self, offset_seconds: f64, should_play: bool) -> Result<(), String> {
        let Some(path) = self.path.clone() else {
            return Ok(());
        };

        let file = File::open(&path).map_err(|error| format!("Cannot open file: {error}"))?;
        let decoder = Decoder::new(BufReader::new(file))
            .map_err(|error| format!("Cannot decode audio: {error}"))?;

        let total_duration = decoder
            .total_duration()
            .map(|duration| duration.as_secs_f64())
            .unwrap_or(0.0);
        self.duration = total_duration;

        let clamped_offset = if total_duration > 0.0 {
            offset_seconds.max(0.0).min(total_duration)
        } else {
            offset_seconds.max(0.0)
        };
        self.position_offset = clamped_offset;

        self.sink.stop();
        self.sink = Sink::try_new(&self.handle)
            .map_err(|error| format!("Cannot recreate audio sink: {error}"))?;
        self.sink.set_volume(self.volume);
        self.sink
            .append(decoder.skip_duration(Duration::from_secs_f64(clamped_offset)));

        if should_play {
            self.sink.play();
            self.paused = false;
        } else {
            self.sink.pause();
            self.paused = true;
        }

        Ok(())
    }

    fn position(&self) -> f64 {
        if self.path.is_none() {
            return 0.0;
        }

        let absolute = self.position_offset + self.sink.get_pos().as_secs_f64();
        if self.duration > 0.0 {
            absolute.min(self.duration)
        } else {
            absolute
        }
    }
}
