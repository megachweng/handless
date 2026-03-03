use super::{VadFrame, VoiceActivityDetector};
use anyhow::Result;

pub struct SmoothedVad {
    inner_vad: Box<dyn VoiceActivityDetector>,
    hangover_frames: usize,
    onset_frames: usize,

    // Pre-allocated flat ring buffer for frame storage.
    // Holds up to `max_frames` frames of `frame_size` samples each.
    // Lazily initialized on the first push_frame call; after that,
    // the audio thread performs zero heap allocations.
    ring_buf: Vec<f32>,
    frame_size: usize,
    max_frames: usize,
    frame_count: usize,
    write_head: usize,

    hangover_counter: usize,
    onset_counter: usize,
    in_speech: bool,

    temp_out: Vec<f32>,
}

impl SmoothedVad {
    pub fn new(
        inner_vad: Box<dyn VoiceActivityDetector>,
        prefill_frames: usize,
        hangover_frames: usize,
        onset_frames: usize,
    ) -> Self {
        let max_frames = prefill_frames + 1;
        Self {
            inner_vad,
            hangover_frames,
            onset_frames,
            ring_buf: Vec::new(),
            frame_size: 0,
            max_frames,
            frame_count: 0,
            write_head: 0,
            hangover_counter: 0,
            onset_counter: 0,
            in_speech: false,
            temp_out: Vec::new(),
        }
    }
}

impl VoiceActivityDetector for SmoothedVad {
    fn push_frame<'a>(&'a mut self, frame: &'a [f32]) -> Result<VadFrame<'a>> {
        let fs = frame.len();

        // Lazy-initialize the flat ring buffer on the first frame.
        // After this, no further allocations occur on the audio thread.
        if self.frame_size == 0 {
            self.frame_size = fs;
            self.ring_buf = vec![0.0f32; self.max_frames * fs];
            self.temp_out = Vec::with_capacity(self.max_frames * fs);
        }

        // 1. Write the incoming frame into the ring buffer (overwrites oldest when full)
        let start = self.write_head * fs;
        self.ring_buf[start..start + fs].copy_from_slice(frame);
        self.write_head = (self.write_head + 1) % self.max_frames;
        if self.frame_count < self.max_frames {
            self.frame_count += 1;
        }

        // 2. Delegate to the wrapped boolean VAD
        let is_voice = self.inner_vad.is_voice(frame)?;

        match (self.in_speech, is_voice) {
            // Potential start of speech - need to accumulate onset frames
            (false, true) => {
                self.onset_counter += 1;
                if self.onset_counter >= self.onset_frames {
                    // We have enough consecutive voice frames to trigger speech
                    self.in_speech = true;
                    self.hangover_counter = self.hangover_frames;
                    self.onset_counter = 0; // Reset for next time

                    // Collect all buffered frames (oldest → newest) into temp_out
                    self.temp_out.clear();
                    let oldest = if self.frame_count == self.max_frames {
                        self.write_head // write_head just advanced past newest, so it points to oldest
                    } else {
                        0
                    };
                    for i in 0..self.frame_count {
                        let idx = (oldest + i) % self.max_frames;
                        let begin = idx * fs;
                        self.temp_out
                            .extend_from_slice(&self.ring_buf[begin..begin + fs]);
                    }
                    Ok(VadFrame::Speech(&self.temp_out))
                } else {
                    // Not enough frames yet, still silence
                    Ok(VadFrame::Noise)
                }
            }

            // Ongoing Speech
            (true, true) => {
                self.hangover_counter = self.hangover_frames;
                Ok(VadFrame::Speech(frame))
            }

            // End of Speech or interruption during onset phase
            (true, false) => {
                if self.hangover_counter > 0 {
                    self.hangover_counter -= 1;
                    Ok(VadFrame::Speech(frame))
                } else {
                    self.in_speech = false;
                    Ok(VadFrame::Noise)
                }
            }

            // Silence or broken onset sequence
            (false, false) => {
                self.onset_counter = 0; // Reset onset counter on silence
                Ok(VadFrame::Noise)
            }
        }
    }

    fn reset(&mut self) {
        self.frame_count = 0;
        self.write_head = 0;
        self.hangover_counter = 0;
        self.onset_counter = 0;
        self.in_speech = false;
        self.temp_out.clear();
    }
}
