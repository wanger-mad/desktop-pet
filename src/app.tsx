import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Eye, EyeOff, Gauge, Heart, RotateCcw, Settings2, X } from "lucide-preact";
import { useEffect, useRef, useState } from "preact/hooks";
import type { PetSettings, PetState } from "./types";

const lines = {
  calm: ["我在这儿。", "窗边的风刚刚好。", "忙完记得伸个懒腰。"],
  happy: ["嘿嘿，再摸一下！", "今天也一起加油。", "抓到你的鼠标啦！"],
  sleepy: ["让我眯一小会儿……", "晚一点再玩吧。", "困困。"],
  annoyed: ["慢一点嘛。", "我需要一点安静。", "先让我缓一缓。"]
};

function SettingsPanel({ state, onState }: { state: PetState; onState: (s: PetState) => void }) {
  const update = async (patch: Partial<PetSettings>) => onState(await invoke<PetState>("update_settings", { patch }));
  return <main class="settings-shell">
    <header><div><span class="eyebrow">LOCAL COMPANION</span><h1>小团子设置</h1></div><button class="icon-button" title="关闭设置" onClick={() => getCurrentWindow().hide()}><X size={18}/></button></header>
    <section class="setting-section">
      <h2>外观</h2>
      <div class="setting-row"><div><strong>桌宠尺寸</strong><span>更改后立即生效</span></div><div class="segments">{[150,300,420,500].map(size => <button class={state.settings.size === size ? "active" : ""} onClick={() => update({ size: size as PetSettings["size"] })}>{size}</button>)}</div></div>
      <label class="setting-row"><div><strong>对话气泡</strong><span>显示本地台词和互动反馈</span></div><input type="checkbox" checked={state.settings.showBubble} onChange={e => update({ showBubble: e.currentTarget.checked })}/></label>
      <label class="setting-row"><div><strong>始终置顶</strong><span>让小团子保持在其他窗口上方</span></div><input type="checkbox" checked={state.settings.alwaysOnTop} onChange={e => update({ alwaysOnTop: e.currentTarget.checked })}/></label>
    </section>
    <section class="setting-section">
      <h2>互动</h2>
      <label class="setting-row"><div><strong>点击穿透</strong><span>开启后可从托盘菜单恢复互动</span></div><input type="checkbox" checked={state.settings.clickThrough} onChange={e => update({ clickThrough: e.currentTarget.checked })}/></label>
    </section>
    <section class="setting-section">
      <h2>当前状态</h2>
      <div class="stats"><span><Heart size={16}/> 好感度 <b>{state.affection}</b></span><span><Gauge size={16}/> 精力 <b>{state.energy}</b></span><span>互动 <b>{state.interactions}</b></span></div>
      <button class="secondary" onClick={async () => onState(await invoke<PetState>("reset_state"))}><RotateCcw size={16}/>重置状态</button>
    </section>
  </main>;
}

function Pet({ state, onState }: { state: PetState; onState: (s: PetState) => void }) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const imageRef = useRef<HTMLImageElement | null>(null);
  const [bubble, setBubble] = useState("我在这儿。");
  const [pressed, setPressed] = useState(false);
  const drag = useRef({ x: 0, y: 0, started: false });
  const size = state.settings.size;

  useEffect(() => {
    const canvas = canvasRef.current!;
    const dpr = window.devicePixelRatio;
    canvas.width = Math.round(size * dpr);
    canvas.height = Math.round(size * dpr);
    const ctx = canvas.getContext("2d")!;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    const draw = (img: HTMLImageElement) => {
      ctx.clearRect(0, 0, size, size);
      ctx.drawImage(img, size * .08, size * .09, size * .84, size * .84);
    };
    const asset = state.mood === "happy" ? "happy" : state.mood === "sleepy" ? "sleepy" : state.mood === "annoyed" ? "mad" : "normal_idle";
    if (imageRef.current?.dataset.asset === asset && imageRef.current.complete) {
      draw(imageRef.current);
      return;
    }
    const img = new Image();
    img.dataset.asset = asset;
    img.onload = () => { imageRef.current = img; draw(img); };
    img.src = `/pet/${asset}.webp`;
  }, [size, state.mood]);

  const interact = async (kind: "head" | "belly" | "tap") => {
    const next = await invoke<PetState>("interact", { kind });
    onState(next);
    const pool = lines[next.mood];
    setBubble(pool[next.interactions % pool.length]);
  };

  return <main class="pet-shell" style={{ width: size, height: size }}>
    {state.settings.showBubble && <div class="bubble">{bubble}</div>}
    <canvas ref={canvasRef} class={pressed ? "pressed" : ""}
      onPointerDown={e => { setPressed(true); drag.current = { x: e.clientX, y: e.clientY, started: false }; e.currentTarget.setPointerCapture(e.pointerId); }}
      onPointerMove={async e => { if (!pressed || drag.current.started) return; if (Math.hypot(e.clientX-drag.current.x,e.clientY-drag.current.y)>8) { drag.current.started = true; setPressed(false); await getCurrentWindow().startDragging(); } }}
      onPointerUp={e => { setPressed(false); if (!drag.current.started) interact(e.clientY < size*.52 ? "head" : e.clientY > size*.66 ? "belly" : "tap"); }}
      onPointerCancel={() => { setPressed(false); drag.current.started = false; }}
    />
    <button class="pet-action settings" title="设置" onClick={() => invoke("open_settings")}><Settings2 size={16}/></button>
    <button class="pet-action passthrough" title={state.settings.clickThrough ? "关闭点击穿透" : "开启点击穿透"} onClick={() => invoke("update_settings", { patch: { clickThrough: !state.settings.clickThrough } }).then(s => onState(s as PetState))}>{state.settings.clickThrough ? <EyeOff size={16}/> : <Eye size={16}/>}</button>
  </main>;
}

export function App() {
  const [state, setState] = useState<PetState | null>(null);
  const settingsWindow = new URLSearchParams(location.search).has("settings");
  useEffect(() => { invoke<PetState>("get_state").then(setState); }, []);
  if (!state) return <div class="loading">正在唤醒小团子…</div>;
  return settingsWindow ? <SettingsPanel state={state} onState={setState}/> : <Pet state={state} onState={setState}/>;
}
