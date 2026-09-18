export type Mood = "calm" | "happy" | "sleepy" | "annoyed";

export interface PetSettings {
  size: 150 | 300 | 420 | 500;
  showBubble: boolean;
  clickThrough: boolean;
  alwaysOnTop: boolean;
  autoStart: boolean;
  focusMinutes: number;
  shortBreakMinutes: number;
  longBreakMinutes: number;
  longBreakEvery: number;
  pomodoroAutoStart: boolean;
  pomodoroBubble: boolean;
}

export interface PomodoroState {
  phase: "idle" | "focus" | "shortBreak" | "longBreak" | "paused" | "completed";
  running: boolean;
  remainingSeconds: number;
  round: number;
  totalCompleted: number;
}

export interface PetState {
  version: number;
  affection: number;
  energy: number;
  hunger: number;
  boredom: number;
  interactions: number;
  mood: Mood;
  settings: PetSettings;
  pomodoro: PomodoroState;
}
