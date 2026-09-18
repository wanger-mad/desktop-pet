export type Mood = "calm" | "happy" | "sleepy" | "annoyed";

export interface PetSettings {
  size: 150 | 300 | 420 | 500;
  showBubble: boolean;
  clickThrough: boolean;
  alwaysOnTop: boolean;
  autoStart: boolean;
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
}
