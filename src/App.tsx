import { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { X, Trash2, Search } from "lucide-react";
import { ClipboardEntry, ClipboardEntryData } from "./types";
import Sidebar from "./components/Sidebar";
import ClipboardCard from "./components/ClipboardCard";
import { Button } from "./components/ui/button";
import { Input } from "./components/ui/input";

function App() {
  const [clipboardEvents, setClipboardEvents] = useState<ClipboardEntry[]>([]);
  const [activeCategory, setActiveCategory] = useState<"all" | "text" | "images">("all");
  const [searchQuery, setSearchQuery] = useState("");

  const textCount = useMemo(() => clipboardEvents.filter((e) => e.isText()).length, [clipboardEvents]);
  const imageCount = useMemo(() => clipboardEvents.filter((e) => e.isImage()).length, [clipboardEvents]);

  const filteredEvents = useMemo(() => {
    let filtered: ClipboardEntry[];
    if (activeCategory === "text") {
      filtered = clipboardEvents.filter((e) => e.isText());
    } else if (activeCategory === "images") {
      filtered = clipboardEvents.filter((e) => e.isImage());
    } else {
      filtered = clipboardEvents;
    }

    return filtered;
  }, [clipboardEvents, activeCategory]);

  useEffect(() => {
    console.log("[INIT] React App mounting at:", new Date().toISOString());

    if (typeof (window as any).__TAURI_INTERNALS__ === "undefined") {
      console.error("[ERROR] ⚠️ NOT RUNNING IN TAURI CONTEXT ⚠️");
      return;
    }

    let unlistenClipboardChanged: any;
    let unlistenClipboardDeleted: any;

    const listenerInstanceId = Math.random().toString(36).substring(7);
    console.log("[MOUNT] Registering clipboard-changed listener:", listenerInstanceId);

    listen<ClipboardEntryData>("clipboard-changed", (event) => {
      console.log(`[EVENT-${listenerInstanceId}] clipboard-changed received:`, event.payload);
      const entry = new ClipboardEntry(event.payload);
      setClipboardEvents((prev) => {
        const isDuplicate = prev.some((e) => e.id === entry.id);
        if (isDuplicate) {
          console.log(`[EVENT-${listenerInstanceId}] Duplicate entry detected (id: ${entry.id}), skipping`);
          return prev;
        }
        console.log(`[EVENT-${listenerInstanceId}] Adding entry (id: ${entry.id}), current count:`, prev.length);
        return [entry, ...prev];
      });
    }).then((fn) => {
      unlistenClipboardChanged = fn;
      console.log("[MOUNT] ✓ clipboard-changed listener registered:", listenerInstanceId);
    });

    listen<number>("clipboard-deleted", (event) => {
      console.log("[EVENT] clipboard-deleted received:", event.payload);
      setClipboardEvents((prev) => prev.filter((e) => e.id !== event.payload));
    }).then((fn) => {
      unlistenClipboardDeleted = fn;
      console.log("[MOUNT] ✓ clipboard-deleted listener registered");
    });

    invoke<ClipboardEntryData[]>("load_clipboard_events_at_startup")
      .then((entries) => {
        console.log("[MOUNT] Loaded entries:", entries);
        const clipboardEntries = entries.map((e) => new ClipboardEntry(e));
        setClipboardEvents(clipboardEntries);
        console.log("[MOUNT] ✓ Initial data loaded:", clipboardEntries.length);
      })
      .catch((error) => {
        console.error("[ERROR] Failed to load initial data:", error);
      });

    return () => {
      if (unlistenClipboardChanged) {
        unlistenClipboardChanged();
        console.log("[CLEANUP] clipboard-changed listener removed");
      }
      if (unlistenClipboardDeleted) {
        unlistenClipboardDeleted();
        console.log("[CLEANUP] clipboard-deleted listener removed");
      }
    };
  }, []);

  const handleDelete = async (item: ClipboardEntry) => {
    console.log("[DELETE] Deleting item:", item);
    setClipboardEvents((prev) => prev.filter((e) => e.id !== item.id));
    invoke("delete_clipboard_entry", { id: item.id });
  };

  const handlePaste = async (item: ClipboardEntry) => {
    try {
      if (item.isText()) {
        await navigator.clipboard.writeText(item.text);
        console.log("[PASTE] Pasted text:", item.text);
      } else if (item.isImage()) {
        console.log("[PASTE] Image paste not yet implemented");
      }
    } catch (error) {
      console.error("[ERROR] Failed to paste:", error);
    }
  };

  const handleClearAll = async () => {
    console.log("[CLEAR] Clearing all entries");
    try {
      await invoke("clear_clipboard_history");
      setClipboardEvents([]);
    } catch (error) {
      console.error("[ERROR] Failed to clear:", error);
    }
  };

  const handleCloseWindow = async () => {
    try {
      await invoke("hide_window");
    } catch (error) {
      console.error("[ERROR] Failed to hide window:", error);
    }
  };

  return (
    <div className="flex h-screen overflow-hidden bg-background">
      <Sidebar
        totalCount={clipboardEvents.length}
        textCount={textCount}
        imageCount={imageCount}
        onCategoryChange={setActiveCategory}
        onToggle={() => {}}
      />

      <main className="flex flex-1 flex-col overflow-hidden">
        {/* Header */}
        <header className="flex items-center justify-between border-b px-4 py-3">
          <h2 className="text-lg font-semibold">
            {activeCategory === "all" ? "Clipboard History" : activeCategory === "text" ? "Text Items" : "Images"}
          </h2>
          <div className="flex items-center gap-2">
            <Button variant="outline" size="sm" onClick={handleClearAll} className="gap-2">
              <Trash2 className="h-4 w-4" />
              Clear All
            </Button>
            <Button variant="ghost" size="icon" className="h-8 w-8" onClick={handleCloseWindow} title="Close window">
              <X className="h-4 w-4" />
            </Button>
          </div>
        </header>

        {/* Search Bar */}
        {activeCategory === "text" && (
          <div className="border-b px-4 py-3">
            <div className="relative">
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                type="text"
                placeholder="Search text items..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="pl-9 pr-9"
              />
              {searchQuery && (
                <Button
                  variant="ghost"
                  size="icon"
                  className="absolute right-1 top-1/2 h-7 w-7 -translate-y-1/2"
                  onClick={() => setSearchQuery("")}
                  title="Clear search"
                >
                  <X className="h-3 w-3" />
                </Button>
              )}
            </div>
          </div>
        )}

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-4">
          {filteredEvents.length === 0 ? (
            <div className="flex h-full flex-col items-center justify-center text-center">
              <p className="text-base font-medium text-foreground">
                {activeCategory === "all"
                  ? "Your clipboard history is empty."
                  : activeCategory === "text"
                  ? "No text items yet."
                  : "No images yet."}
              </p>
              <p className="mt-2 text-sm text-muted-foreground">
                Copy some {activeCategory === "all" ? "text or images" : activeCategory} to get started!
              </p>
            </div>
          ) : (
            <div className="space-y-3">
              {filteredEvents.map((item) => (
                <ClipboardCard key={`${item.id}-${item.timestamp}`} item={item} onDelete={handleDelete} onPaste={handlePaste} />
              ))}
            </div>
          )}
        </div>
      </main>
    </div>
  );
}

export default App;
