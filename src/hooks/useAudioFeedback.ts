import { useRef, useCallback } from "react";

type SoundType = "CLICK" | "ASCEND" | "DESCEND" | "RECORD" | "ERROR";

interface UseAudioFeedbackReturn {
  play: (type: SoundType) => void;
}

/**
 * Web Audio API oscillator-based sound effects.
 * Mirrors the original Pinia store's playFeedbackSound() logic.
 * Uses a singleton AudioContext via useRef to survive StrictMode double-mount.
 */
export function useAudioFeedback(enabled: boolean): UseAudioFeedbackReturn {
  const ctxRef = useRef<AudioContext | null>(null);

  const getCtx = useCallback((): AudioContext => {
    if (!ctxRef.current) {
      ctxRef.current = new (window.AudioContext ||
        (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext)();
    }
    return ctxRef.current;
  }, []);

  const playOsc = useCallback(
    (freq: number, dur: number, gainVal: number, type: OscillatorType = "triangle") => {
      const ctx = getCtx();
      const now = ctx.currentTime;
      const osc = ctx.createOscillator();
      const gain = ctx.createGain();
      osc.type = type;
      osc.frequency.setValueAtTime(freq, now);
      gain.gain.setValueAtTime(gainVal, now);
      gain.gain.exponentialRampToValueAtTime(0.01, now + dur);
      osc.connect(gain);
      gain.connect(ctx.destination);
      osc.start();
      osc.stop(now + dur);
    },
    [getCtx],
  );

  const play = useCallback(
    (type: SoundType) => {
      if (!enabled) return;
      switch (type) {
        case "CLICK":
          playOsc(1200, 0.08, 0.1, "square");
          break;
        case "ASCEND":
          playOsc(440, 0.2, 0.05);
          setTimeout(() => playOsc(880, 0.2, 0.04), 80);
          break;
        case "DESCEND":
          playOsc(660, 0.2, 0.05);
          setTimeout(() => playOsc(330, 0.2, 0.04), 80);
          break;
        case "RECORD":
          playOsc(1000, 0.1, 0.08, "sine");
          break;
        case "ERROR":
          playOsc(200, 0.15, 0.1, "sawtooth");
          setTimeout(() => playOsc(150, 0.2, 0.1, "sawtooth"), 120);
          break;
      }
    },
    [enabled, playOsc],
  );

  return { play };
}
