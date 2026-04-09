#ifndef notch_indicator_bridge_h
#define notch_indicator_bridge_h

#ifdef __cplusplus
extern "C" {
#endif

// Initialize the notch indicator panel (call once during app startup)
void notch_indicator_init(void);

// Update the indicator state: 0=hidden, 1=recording, 2=transcribing
void notch_indicator_update_state(int state);

// Update the audio input level (0.0-1.0), drives the recording dot and waveform
void notch_indicator_update_audio_level(float level);

// Append streaming transcription text (notch expands to show it)
void notch_indicator_update_streaming_text(const char* text);

// Tear down the panel (optional, called on app quit)
void notch_indicator_destroy(void);

#ifdef __cplusplus
}
#endif

#endif /* notch_indicator_bridge_h */
