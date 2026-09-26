import { useEffect, useState } from "react";
import {
  ring,
  ringElapsed,
  ringIndicator,
  ringLabel,
  ringSpinner,
  ringSvg,
  ringText,
  ringTrack,
} from "./GenerationProgress.css.ts";

export interface ProgressRingProps {
  /** What is happening right now, in the teacher's words. */
  label: string;
  /**
   * 0-100 where the fraction is actually known, null where it is not.
   * A null sweeps a fixed arc rather than inventing a number.
   */
  percent?: number | null;
  /** Milliseconds since the wait started. */
  elapsedMs: number;
  ariaLabel: string;
}

/** Seconds while that reads naturally, minutes once it does not. */
export function formatElapsed(ms: number): string {
  const seconds = ms / 1000;
  if (seconds < 60) return `${seconds.toFixed(1)} 秒`;
  return `${Math.floor(seconds / 60)} 分 ${Math.floor(seconds % 60)} 秒`;
}

/**
 * Elapsed milliseconds while `active`, resetting each time it turns on.
 *
 * On-device generation takes tens of seconds, and how long it has already
 * taken is the one thing the teacher can always be told truthfully.
 */
export function useElapsed(active: boolean): number {
  const [elapsed, setElapsed] = useState(0);

  useEffect(() => {
    if (!active) return;
    const started = performance.now();
    setElapsed(0);
    const timer = setInterval(() => {
      setElapsed(performance.now() - started);
    }, 100);
    return () => clearInterval(timer);
  }, [active]);

  return active ? elapsed : 0;
}

const RADIUS = 12;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;

/**
 * One wait, drawn where its result will appear.
 *
 * This is deliberately a ring rather than a full-width bar: it is shown only
 * while waiting, and a bar inserted above the content moved everything below
 * it each time a generation started and finished.
 */
export function ProgressRing({
  label,
  percent,
  elapsedMs,
  ariaLabel,
}: ProgressRingProps) {
  const known = percent != null && Number.isFinite(percent);
  const value = known ? Math.min(Math.max(percent, 0), 100) : null;

  return (
    <div className={ring}>
      <svg
        className={ringSvg}
        viewBox="0 0 28 28"
        role="progressbar"
        aria-label={ariaLabel}
        aria-valuenow={value ?? undefined}
        aria-valuemin={value != null ? 0 : undefined}
        aria-valuemax={value != null ? 100 : undefined}
        aria-valuetext={value == null ? "進捗は測定できません" : undefined}
      >
        <circle className={ringTrack} cx="14" cy="14" r={RADIUS} />
        <circle
          className={`${ringIndicator}${value == null ? ` ${ringSpinner}` : ""}`}
          cx="14"
          cy="14"
          r={RADIUS}
          strokeDasharray={
            value == null
              ? `${CIRCUMFERENCE * 0.25} ${CIRCUMFERENCE}`
              : CIRCUMFERENCE
          }
          strokeDashoffset={
            value == null ? 0 : CIRCUMFERENCE * (1 - value / 100)
          }
        />
      </svg>
      <span className={ringText}>
        <span className={ringLabel}>{label}</span>
        <span className={ringElapsed}>経過 {formatElapsed(elapsedMs)}</span>
      </span>
    </div>
  );
}
