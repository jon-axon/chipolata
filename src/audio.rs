use rodio::{source::SineWave, OutputStream, Sink};

/// Simple struct to represent an audio stream, with a sink that can be paused and resumed
/// as required
pub(crate) struct Audio {
    _stream: OutputStream,
    sink: Sink,
}

impl Audio {
    /// Constructor that returns an [Audio] instance whose audio source is a basis sinewave
    /// at the pitch 440hz (A).  The stream begins in a paused state
    pub(crate) fn new() -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink: Sink = Sink::try_new(&stream_handle).unwrap();
        let audio: Audio = Audio { _stream, sink };
        audio.sink.append(SineWave::new(440.0));
        audio.sink.pause();
        audio
    }

    /// Resumes playback if the stream is paused
    pub(crate) fn play(&self) -> () {
        self.sink.play();
    }

    /// Pauses playback if the stream is playing
    pub(crate) fn pause(&self) -> () {
        self.sink.pause();
    }

    /// Returns true if the stream is currently paused, otherwise false
    pub(crate) fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }
}
