use rodio::{source::SineWave, OutputStream, Sink};

pub(crate) struct Audio {
    _stream: OutputStream,
    sink: Sink,
}

impl Audio {
    pub(crate) fn new() -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink: Sink = Sink::try_new(&stream_handle).unwrap();
        let audio: Audio = Audio { _stream, sink };
        audio.sink.append(SineWave::new(440.0));
        audio.sink.pause();
        audio
    }

    pub(crate) fn play(&self) -> () {
        self.sink.play();
    }

    pub(crate) fn pause(&self) -> () {
        self.sink.pause();
    }

    pub(crate) fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }
}
