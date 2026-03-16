import { listen } from "@tauri-apps/api/event";
import React, {
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";

import "./RecordingOverlay.css";
import i18n, { syncLanguageFromSettings } from "@/i18n";
import { getLanguageDirection } from "@/lib/utils/rtl";
import { commands, type ActivationMode } from "@/bindings";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { X, Check } from "@phosphor-icons/react";

type OverlayState = "recording" | "transcribing" | "processing";

const CHANNEL_COUNT = 11;
const THINKING_DOT_COUNT = 6;
const ATTACK_SPEED = 0.4;
const DECAY_SPEED = 0.3;
const STREAMING_WIDTH = 300;
const STREAMING_LINE_HEIGHT = 18;
const MAX_LINES = 5;
const OVERLAY_PADDING = 12;
const BUTTON_AREA_WIDTH = 20; // px per side for cancel/confirm buttons
const BUBBLE_CSS_WIDTH = 280; // must match .streaming-text-bubble { width }
const BUBBLE_CHROME = 18; // 8+8 padding + 1+1 border on .streaming-text-bubble
const BUBBLE_GAP = 6; // must match .overlay-wrapper { gap }
const BUBBLE_MARGIN = 10; // window margin around bubble for shadows

// Apple-style waveform: thick rounded bars that stretch vertically
const BAR_WIDTH = 2.5;
const BAR_GAP = 2;
const BAR_MIN_HEIGHT = 3;
const BAR_MAX_HEIGHT = 20;
const BAR_RADIUS = BAR_WIDTH / 2;

// Per-bar vertical wobble for organic misalignment
const BAR_WOBBLE = [
  { phase: 0, freq: 0.7, amp: 1.2 },
  { phase: 1.3, freq: 1.0, amp: 0.8 },
  { phase: 0.6, freq: 0.85, amp: 1.0 },
  { phase: 2.1, freq: 1.2, amp: 0.9 },
  { phase: 1.5, freq: 0.95, amp: 1.1 },
  { phase: 0.9, freq: 1.15, amp: 0.85 },
  { phase: 1.8, freq: 0.75, amp: 1.0 },
];

// Which smoothed audio channels drive each bar
const BAR_CHANNELS = [
  [0, 1],
  [1, 2, 3],
  [3, 4],
  [4, 5, 6],
  [6, 7],
  [7, 8, 9],
  [9, 10],
];

// Per-bar amplitude envelope — gently center-weighted
const BAR_ENVELOPE = [0.45, 0.7, 0.85, 1.0, 0.85, 0.7, 0.45];
const BAR_COUNT = BAR_WOBBLE.length;

// Layered sine noise — irrational frequency ratios create non-repeating organic motion
function organicNoise(t: number, seed: number): number {
  return (
    Math.sin(t * 1.0 + seed) * 0.5 +
    Math.sin(t * 2.31 + seed * 1.73) * 0.3 +
    Math.sin(t * 3.67 + seed * 2.19) * 0.2
  );
}

// Non-harmonic durations & staggered delays so dots never sync
const THINKING_DURATIONS = [1.3, 1.7, 1.1, 1.5, 1.9, 1.25];
const THINKING_DELAYS = [0, 0.4, 0.15, 0.65, 0.3, 0.8];

const RecordingOverlay: React.FC = () => {
  const [isVisible, setIsVisible] = useState(false);
  const [state, setState] = useState<OverlayState>("recording");
  const [activationMode, setActivationMode] =
    useState<ActivationMode>("hold_or_toggle");
  const [streamingText, setStreamingText] = useState("");
  const [overlayPosition, setOverlayPosition] = useState<"top" | "bottom">(
    "top",
  );
  const [progress, setProgress] = useState(0);

  // Accent RGB resolved from CSS variable for use in inline box-shadow strings
  const accentRgbRef = useRef("239, 111, 47");
  useEffect(() => {
    accentRgbRef.current = getComputedStyle(document.documentElement)
      .getPropertyValue("--color-accent-rgb")
      .trim();
  }, []);

  // Audio level smoothing via rAF — bypasses React rendering
  const targetLevelsRef = useRef<number[]>(Array(CHANNEL_COUNT).fill(0));
  const currentLevelsRef = useRef<number[]>(Array(CHANNEL_COUNT).fill(0));
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const canvasCtxRef = useRef<CanvasRenderingContext2D | null>(null);
  const peakEnergyRef = useRef(0.01); // adaptive ceiling for relative scaling
  const lastFrameTimeRef = useRef(0);
  const rafIdRef = useRef<number>(0);
  const frameCountRef = useRef(0);
  const overlayRef = useRef<HTMLDivElement>(null);

  // Progress bar refs
  const progressIntervalRef = useRef<ReturnType<typeof setInterval> | null>(
    null,
  );
  const progressStartTimeRef = useRef<number>(0);
  const hideTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Word tracking for per-word animation
  const prevWordCountRef = useRef(0);
  const [words, setWords] = useState<{ text: string; isNew: boolean }[]>([]);

  // Streaming text height measurement
  const streamingTextRef = useRef<HTMLDivElement>(null);
  const [contentHeight, setContentHeight] = useState(0);
  const direction = getLanguageDirection(i18n.language);

  const hasStreamingText = state === "recording" && streamingText.length > 0;
  const isPostProcessing = state === "transcribing" || state === "processing";
  // Buttons visible & interactive only in toggle mode
  const showButtons = state === "recording" && activationMode === "toggle";
  const buttonsExtra = showButtons ? BUTTON_AREA_WIDTH * 2 : 0;
  // Toggle mode: text floats above waveform bars instead of replacing them
  const showTextAboveBars = hasStreamingText && activationMode === "toggle";

  // Compute overlay dimensions — pill never changes shape in toggle mode
  const overlayWidth = (() => {
    if (!isVisible) return 33;
    if (hasStreamingText && !showTextAboveBars) return STREAMING_WIDTH + buttonsExtra;
    return 70 + buttonsExtra;
  })();

  const overlayHeight = (() => {
    if (!isVisible) return 33;
    if (hasStreamingText && !showTextAboveBars && contentHeight > 0) {
      const maxTextHeight = STREAMING_LINE_HEIGHT * MAX_LINES;
      const clampedHeight = Math.min(contentHeight, maxTextHeight);
      return Math.max(33, clampedHeight + OVERLAY_PADDING);
    }
    return 33;
  })();

  const overlayRadius = hasStreamingText && !showTextAboveBars ? 14 : 999;
  const buttonRadius = hasStreamingText && !showTextAboveBars ? 8 : 13;

  // Track words (runs before paint to avoid flicker)
  useLayoutEffect(() => {
    if (streamingText) {
      const allWords = streamingText.split(/\s+/).filter(Boolean);
      const prevCount = prevWordCountRef.current;
      setWords(
        allWords.map((w, i) => ({
          text: w,
          isNew: i >= prevCount && prevCount > 0,
        })),
      );
      prevWordCountRef.current = allWords.length;
    } else {
      setWords([]);
      prevWordCountRef.current = 0;
      setContentHeight(0);
    }
  }, [streamingText]);

  // rAF loop: canvas-based vertical bars driven by smoothed audio energy
  const animateWaveform = useCallback((timestamp: number) => {
    const lastTime = lastFrameTimeRef.current;
    const dt =
      lastTime > 0 ? Math.min((timestamp - lastTime) / 1000, 0.05) : 1 / 60;
    lastFrameTimeRef.current = timestamp;

    const current = currentLevelsRef.current;
    const target = targetLevelsRef.current;
    const frame = frameCountRef.current++;

    // Asymmetric LERP: snappy attack, slow lingering decay
    // Normalize to 60fps so behavior is consistent across refresh rates
    const steps = dt * 60;
    for (let i = 0; i < CHANNEL_COUNT; i++) {
      const base = target[i] > current[i] ? ATTACK_SPEED : DECAY_SPEED;
      const speed = 1 - Math.pow(1 - base, steps);
      current[i] += (target[i] - current[i]) * speed;
    }

    // Adaptive peak: rise instantly to new max, decay slowly so quiet
    // speakers still get full-height bars relative to their own volume
    const maxChannel = Math.max(...current);
    if (maxChannel > peakEnergyRef.current) {
      peakEnergyRef.current = maxChannel;
    } else {
      peakEnergyRef.current *= Math.pow(0.5, dt); // halves in 1s
      peakEnergyRef.current = Math.max(peakEnergyRef.current, 0.01);
    }
    const peak = peakEnergyRef.current;

    const avgEnergy = current.reduce((a, b) => a + b, 0) / current.length;
    const rgb = accentRgbRef.current;

    // Draw Apple-style vertical bars
    const canvas = canvasRef.current;
    if (canvas) {
      if (!canvasCtxRef.current) {
        canvasCtxRef.current = canvas.getContext("2d");
      }
      const ctx = canvasCtxRef.current;
      if (ctx) {
        const dpr = window.devicePixelRatio || 1;
        const cssW = canvas.clientWidth;
        const cssH = canvas.clientHeight;

        if (cssW === 0 || cssH === 0) {
          rafIdRef.current = requestAnimationFrame(animateWaveform);
          return;
        }

        const bufW = Math.round(cssW * dpr);
        const bufH = Math.round(cssH * dpr);
        if (canvas.width !== bufW || canvas.height !== bufH) {
          canvas.width = bufW;
          canvas.height = bufH;
        }

        ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
        ctx.clearRect(0, 0, cssW, cssH);

        const centerY = cssH / 2;
        const totalW = BAR_COUNT * BAR_WIDTH + (BAR_COUNT - 1) * BAR_GAP;
        const startX = (cssW - totalW) / 2;

        for (let i = 0; i < BAR_COUNT; i++) {
          const chs = BAR_CHANNELS[i];
          const raw =
            chs.reduce((sum, ch) => sum + current[ch], 0) / chs.length;
          const energy = raw / peak; // normalize to recent peak

          // Organic height modulation via layered incommensurate sines
          const activity = Math.min(energy * 2.5, 1.0);
          const heightMod =
            1.0 + organicNoise(timestamp * 0.0012, i * 1.7) * 0.3 * activity;

          const scaled = energy * BAR_ENVELOPE[i] * heightMod;
          const boosted = Math.min(1, scaled * 1.3);
          const barH =
            BAR_MIN_HEIGHT +
            Math.pow(boosted, 0.8) * (BAR_MAX_HEIGHT - BAR_MIN_HEIGHT);

          // Vertical wobble scales with energy for organic movement
          const w = BAR_WOBBLE[i];
          const wobbleAmt = Math.min(energy * 2.5, 1.0);
          const yOff =
            Math.sin(timestamp * 0.001 * w.freq + w.phase) * w.amp * wobbleAmt;

          const x = startX + i * (BAR_WIDTH + BAR_GAP);
          const y = centerY - barH / 2 + yOff;

          // Per-bar glow
          ctx.shadowBlur = 3 + energy * 10;
          ctx.shadowColor = `rgba(${rgb}, ${0.15 + energy * 0.5})`;

          const alpha = 0.5 + energy * 0.45;
          ctx.fillStyle = `rgba(${rgb}, ${alpha})`;
          ctx.beginPath();
          ctx.roundRect(x, y, BAR_WIDTH, barH, BAR_RADIUS);
          ctx.fill();
        }

        ctx.shadowBlur = 0;
        ctx.shadowColor = "transparent";
      }
    }

    // Ambient overlay glow — breathes when idle, flares with voice
    const overlay = overlayRef.current;
    if (overlay) {
      const breathe = Math.sin(frame * 0.02) * 0.5 + 0.5;
      const idleAmbient = avgEnergy < 0.05 ? breathe * 0.12 : 0;
      const e = Math.max(Math.min(1, avgEnergy * 3), idleAmbient);
      const glowBlur = 8 + e * 18;
      const glowSpread = e * 4;
      const glowAlpha = e * 0.3;
      const innerAlpha = 0.03 + e * 0.06;
      overlay.style.filter = [
        `drop-shadow(0 0 ${glowBlur + glowSpread}px rgba(${rgb}, ${glowAlpha}))`,
        "drop-shadow(0 4px 24px rgba(0, 0, 0, 0.45))",
        "drop-shadow(0 0 0.5px rgba(0, 0, 0, 0.5))",
      ].join(" ");
      overlay.style.boxShadow = [
        `inset 0 1px 0 rgba(${rgb}, 0.04)`,
        `inset 0 0 16px rgba(${rgb}, ${innerAlpha})`,
      ].join(", ");
    }

    rafIdRef.current = requestAnimationFrame(animateWaveform);
  }, []);

  // Start/stop rAF loop based on visibility + recording state
  useEffect(() => {
    if (isVisible && state === "recording") {
      rafIdRef.current = requestAnimationFrame(animateWaveform);
    } else if (overlayRef.current) {
      overlayRef.current.style.filter = "";
      overlayRef.current.style.boxShadow = "";
    }
    return () => cancelAnimationFrame(rafIdRef.current);
  }, [isVisible, state, animateWaveform]);

  const clearProgressInterval = useCallback(() => {
    if (progressIntervalRef.current) {
      clearInterval(progressIntervalRef.current);
      progressIntervalRef.current = null;
    }
  }, []);

  // Progress bar simulation for post-processing
  useEffect(() => {
    if (isPostProcessing && isVisible) {
      setProgress(0);
      progressStartTimeRef.current = Date.now();

      progressIntervalRef.current = setInterval(() => {
        const elapsed = (Date.now() - progressStartTimeRef.current) / 1000;
        // Ease-out curve: approaches 0.85 over ~8 seconds
        const t = Math.min(elapsed / 8, 1);
        const eased = 1 - Math.pow(1 - t, 2.5);
        setProgress(eased * 0.85);
      }, 50);

      return clearProgressInterval;
    }
  }, [isPostProcessing, isVisible, clearProgressInterval]);

  // Measure streaming text height + auto-scroll to show latest content
  useLayoutEffect(() => {
    if (streamingTextRef.current && hasStreamingText) {
      setContentHeight(streamingTextRef.current.scrollHeight);
      streamingTextRef.current.scrollTop =
        streamingTextRef.current.scrollHeight;
    }
  }, [words, hasStreamingText]);

  // Resize overlay window when the text bubble appears/grows so the
  // window tightly fits the visible content. Skip initial show (Rust
  // handles that) to avoid position jumps on quick taps.
  useEffect(() => {
    if (!isVisible || !showTextAboveBars || contentHeight <= 0) return;

    const maxText = STREAMING_LINE_HEIGHT * MAX_LINES;
    const bubbleH = Math.min(contentHeight, maxText) + BUBBLE_CHROME;
    const winW = BUBBLE_CSS_WIDTH + BUBBLE_MARGIN * 2;
    const winH = overlayHeight + bubbleH + BUBBLE_GAP + BUBBLE_MARGIN * 2;
    commands.resizeOverlay(winW, winH);
  }, [isVisible, showTextAboveBars, contentHeight, overlayHeight]);

  useEffect(() => {
    const unlisteners: Array<() => void> = [];

    const setup = async () => {
      unlisteners.push(
        await listen<{
          state: OverlayState;
          position: "top" | "bottom";
          activation_mode: ActivationMode;
        }>("show-overlay", async (event) => {
          const {
            state: overlayState,
            position,
            activation_mode,
          } = event.payload;
          // Reset all state BEFORE any async work so the overlay never
          // renders stale content from the previous session.
          setState(overlayState);
          setOverlayPosition(position);
          setActivationMode(activation_mode);
          setStreamingText("");
          setProgress(0);
          // Cancel any pending hide from a previous session
          if (hideTimeoutRef.current) {
            clearTimeout(hideTimeoutRef.current);
            hideTimeoutRef.current = null;
          }
          // Reset dot levels on new session
          targetLevelsRef.current = Array(CHANNEL_COUNT).fill(0);
          currentLevelsRef.current = Array(CHANNEL_COUNT).fill(0);
          peakEnergyRef.current = 0.01;
          lastFrameTimeRef.current = 0;
          canvasCtxRef.current = null;
          setIsVisible(true);
          // Sync language in the background — doesn't need to block visibility
          syncLanguageFromSettings();
        }),
      );

      unlisteners.push(
        await listen("hide-overlay", () => {
          clearProgressInterval();
          setProgress(1);
          // Delay hide slightly so the user sees 100%
          hideTimeoutRef.current = setTimeout(() => setIsVisible(false), 200);
        }),
      );

      unlisteners.push(
        await listen<number[]>("mic-level", (event) => {
          const newLevels = event.payload;
          for (let i = 0; i < CHANNEL_COUNT; i++) {
            targetLevelsRef.current[i] = newLevels[i] || 0;
          }
        }),
      );

      unlisteners.push(
        await listen<string>("streaming-text", (event) => {
          setStreamingText(event.payload);
        }),
      );

      unlisteners.push(
        await listen<ActivationMode>("update-activation-mode", (event) => {
          setActivationMode(event.payload);
        }),
      );
    };

    setup();

    return () => {
      unlisteners.forEach((fn) => fn());
      if (hideTimeoutRef.current) clearTimeout(hideTimeoutRef.current);
    };
  }, [clearProgressInterval]);

  const handleCancel = useCallback(() => {
    commands.cancelOperation();
  }, []);

  const handleConfirm = useCallback(() => {
    commands.confirmRecording();
  }, []);

  const handleDrag = useCallback((e: React.MouseEvent) => {
    // Don't drag when clicking on buttons
    if ((e.target as HTMLElement).closest("button")) return;
    getCurrentWindow().startDragging();
  }, []);

  const streamingWords = words.map((w, i) => (
    <React.Fragment key={i}>
      {i > 0 && " "}
      <span className={w.isNew ? "word-appear" : undefined}>{w.text}</span>
    </React.Fragment>
  ));

  return (
    <div className={`overlay-wrapper position-${overlayPosition}`}>
      {showTextAboveBars && (
        <div dir={direction} className="streaming-text-bubble">
          <div ref={streamingTextRef} className="bubble-content">
            {streamingWords}
          </div>
        </div>
      )}
      <div
        ref={overlayRef}
        dir={direction}
        onMouseDown={handleDrag}
        style={{
          width: overlayWidth,
          height: overlayHeight,
          borderRadius: overlayRadius,
          clipPath: `inset(0 round ${overlayRadius}px)`,
        }}
        className={`recording-overlay ${isVisible ? "fade-in" : ""} ${showButtons ? "has-buttons" : ""}`}
      >
        {state === "recording" && (
          <>
            <button
              className={`overlay-btn overlay-btn-cancel ${showButtons ? "active" : ""}`}
              style={{ borderRadius: buttonRadius }}
              onClick={handleCancel}
            >
              <X size={12} weight="bold" />
            </button>
            {hasStreamingText && !showTextAboveBars && (
              <div ref={streamingTextRef} className="streaming-text">
                {streamingWords}
              </div>
            )}
            {(!hasStreamingText || showTextAboveBars) && (
              <canvas ref={canvasRef} className="waveform-canvas" />
            )}
            <button
              className={`overlay-btn overlay-btn-confirm ${showButtons ? "active" : ""}`}
              style={{ borderRadius: buttonRadius }}
              onClick={handleConfirm}
            >
              <Check size={12} weight="bold" />
            </button>
          </>
        )}

        {isPostProcessing && (
          <>
            <div
              className="progress-fill"
              style={{ width: `${progress * 100}%` }}
            />
            <div className="thinking-dots">
              {Array.from({ length: THINKING_DOT_COUNT }, (_, i) => (
                <div
                  key={i}
                  className="thinking-dot"
                  style={{
                    animationDuration: `${THINKING_DURATIONS[i]}s`,
                    animationDelay: `${THINKING_DELAYS[i]}s`,
                  }}
                />
              ))}
            </div>
            <button className="postprocess-dismiss" onClick={handleCancel}>
              <X size={10} weight="bold" />
            </button>
          </>
        )}
      </div>
    </div>
  );
};

export default RecordingOverlay;
