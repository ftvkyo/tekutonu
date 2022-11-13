use std::{fs::File, path::PathBuf};

use png::OutputInfo;
use tracing::instrument;

pub struct Texture {
    pub info: OutputInfo,
    pub bytes: Vec<u8>,
}

pub struct TextureLoader {
    path: PathBuf,
}

impl TextureLoader {
    #[instrument]
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    #[instrument(skip(self))]
    pub fn load(&self, name: &str) -> Texture {
        let path_texture = self.path.join(Into::<PathBuf>::into(name));
        let decoder = png::Decoder::new(File::open(path_texture).unwrap());
        let mut reader = decoder.read_info().unwrap();
        let mut bytes = vec![0; reader.output_buffer_size()];

        // Read the next frame. An APNG might contain multiple frames.
        let info = reader.next_frame(&mut bytes).unwrap();

        Texture { info, bytes }
    }
}
