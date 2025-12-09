<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import ClipboardEvent from "../model/clipboard_event";
  import ClipboardEventTable from "../lib/components/table/ClipboardEventTable.svelte";

  let clipboardText = $state("");
  let unlisten;

  let clipboardEvents: ClipboardEvent[] = $state([]);
  let texts: string[] = $state([]);

  onMount(async () => {
    unlisten = await listen('clipboard-changed', (event) => {
      console.log('clipboard-changed', event);
      // clipboardText = event.payload.text;
      // timestamp = event.payload.timestamp;
      // clipboardEvents.push(event.payload);
      // texts.push(event.payload.text);
      clipboardEvents.push(new ClipboardEvent(event.payload.text, event.payload.timestamp));
    });
    await invoke("load_clipboard_events_at_startup");
  });

  async function fetchClipboardText() {
    clipboardText = await invoke("get_clipboard_text");
  }

  async function resetClipboardEvents() {
    clipboardEvents = [];
    await invoke("clear_clipboard_history");
  }

  async function handleCloseWindow() {
    try {
      await invoke("hide_window");
    } catch (error) {
      console.error("Failed to hide window:", error);
    }
  }
</script>

<main class="container">
  <div class="header">
    <h2>Clipboard</h2>
    <button class="close-button" onclick={handleCloseWindow} title="Close window">
      ✕
    </button>
  </div>
  <div>
    <button onclick={resetClipboardEvents}>Reset Clipboard Events</button>
  </div>

  <ClipboardEventTable events={clipboardEvents} />
</main>

<style>
.logo.vite:hover {
  filter: drop-shadow(0 0 2em #747bff);
}

.logo.svelte-kit:hover {
  filter: drop-shadow(0 0 2em #ff3e00);
}

:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

.header {
  display: flex;
  justify-content: center;
  align-items: center;
  position: relative;
  margin-bottom: 1rem;
}

.header h2 {
  margin: 0;
}

.close-button {
  position: absolute;
  right: 20px;
  top: 50%;
  transform: translateY(-50%);
  width: 32px;
  height: 32px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  font-weight: bold;
  border-radius: 50%;
  background-color: #f0f0f0;
  color: #666;
  transition: all 0.2s ease;
}

.close-button:hover {
  background-color: #ff5555;
  color: white;
  border-color: #ff5555;
}

.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: 0.75s;
}

.logo.tauri:hover {
  filter: drop-shadow(0 0 2em #24c8db);
}

.row {
  display: flex;
  justify-content: center;
}

a {
  font-weight: 500;
  color: #646cff;
  text-decoration: inherit;
}

a:hover {
  color: #535bf2;
}

h1 {
  text-align: center;
}

input,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}
button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

input,
button {
  outline: none;
}

#greet-input {
  margin-right: 5px;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  a:hover {
    color: #24c8db;
  }

  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }
  button:active {
    background-color: #0f0f0f69;
  }

  .close-button {
    background-color: #3f3f3f;
    color: #ccc;
  }

  .close-button:hover {
    background-color: #ff5555;
    color: white;
  }
}

</style>
