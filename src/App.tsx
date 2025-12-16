import { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ClipboardEntry, ClipboardEntryData } from "./types";
import Sidebar from "./components/Sidebar";
import ClipboardCard from "./components/ClipboardCard";

function App() {
  const [clipboardEvents, setClipboardEvents] = useState<ClipboardEntry[]>([]);
  const [sidebarExpanded, setSidebarExpanded] = useState(true);
  const [activeCategory, setActiveCategory] = useState<"all" | "text" | "images">("all");
  const [searchQuery, setSearchQuery] = useState("");

  // 카테고리별 카운트 계산
  const textCount = useMemo(() => clipboardEvents.filter((e) => e.isText()).length, [clipboardEvents]);
  const imageCount = useMemo(() => clipboardEvents.filter((e) => e.isImage()).length, [clipboardEvents]);

  // 필터링된 이벤트
  const filteredEvents = useMemo(() => {
    console.log("[FILTER] Active category:", activeCategory);
    console.log("[FILTER] Total events:", clipboardEvents.length);

    let filtered: ClipboardEntry[];
    if (activeCategory === "text") {
      filtered = clipboardEvents.filter((e) => e.isText());
      console.log("[FILTER] Text events:", filtered.length);
    } else if (activeCategory === "images") {
      filtered = clipboardEvents.filter((e) => e.isImage());
      console.log("[FILTER] Image events:", filtered.length);
    } else {
      filtered = clipboardEvents;
      console.log("[FILTER] All events:", filtered.length);
    }

    return filtered;
  }, [clipboardEvents, activeCategory]);

  useEffect(() => {
    console.log("[INIT] ========================================");
    console.log("[INIT] React App mounting at:", new Date().toISOString());

    // Tauri 컨텍스트 확인
    if (typeof (window as any).__TAURI_INTERNALS__ === "undefined") {
      console.error("[ERROR] ⚠️ NOT RUNNING IN TAURI CONTEXT ⚠️");
      return;
    }

    console.log("[MOUNT] ✓ Tauri context confirmed");

    // 각 리스너의 cleanup 함수를 별도로 저장
    let unlistenClipboardChanged: any;
    let unlistenClipboardDeleted: any;

    // clipboard-changed 리스너
    const listenerInstanceId = Math.random().toString(36).substring(7);
    console.log("[MOUNT] Registering clipboard-changed listener:", listenerInstanceId);

    listen<ClipboardEntryData>("clipboard-changed", (event) => {
      console.log(`[EVENT-${listenerInstanceId}] clipboard-changed received:`, event.payload);
      const entry = new ClipboardEntry(event.payload);
      setClipboardEvents((prev) => {
        // 중복 체크: 같은 ID가 이미 존재하면 추가하지 않음
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

    // clipboard-deleted 리스너
    listen<number>("clipboard-deleted", (event) => {
      console.log("[EVENT] clipboard-deleted received:", event.payload);
      setClipboardEvents((prev) => prev.filter((e) => e.id !== event.payload));
    }).then((fn) => {
      unlistenClipboardDeleted = fn;
      console.log("[MOUNT] ✓ clipboard-deleted listener registered");
    });

    // 초기 데이터 로드
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

    // cleanup: 모든 리스너 제거
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
    // 프론트엔드에서 즉시 제거
    setClipboardEvents((prev) => prev.filter((e) => e.timestamp !== item.timestamp));
    // TODO: 백엔드 delete_clipboard_entry 호출
    invoke("delete_clipboard_entry", { id: item.id });
  };

  const handlePaste = async (item: ClipboardEntry) => {
    try {
      if (item.isText()) {
        await navigator.clipboard.writeText(item.text);
        console.log("[PASTE] Pasted text:", item.text);
      } else if (item.isImage()) {
        // TODO: 이미지 paste 구현
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
    <div className={`app-container ${sidebarExpanded ? "" : "sidebar-collapsed"}`}>
      <Sidebar
        totalCount={clipboardEvents.length}
        textCount={textCount}
        imageCount={imageCount}
        onCategoryChange={setActiveCategory}
        onToggle={setSidebarExpanded}
      />

      <main className="main-content">
        <header className="content-header">
          <h2 className="content-title">
            {activeCategory === "all" ? "Clipboard History" : activeCategory === "text" ? "Text Items" : "Images"}
          </h2>
          <div className="header-actions">
            <button className="header-button" onClick={handleClearAll} title="Clear All">
              Clear All
            </button>
            <button className="close-button" onClick={handleCloseWindow} title="Close window">
              ✕
            </button>
          </div>
        </header>

        {activeCategory === "text" && (
          <div className="search-bar">
            <input
              type="text"
              className="search-input"
              placeholder="Search text items..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
            {searchQuery && (
              <button className="search-clear" onClick={() => setSearchQuery("")} title="Clear search">
                ✕
              </button>
            )}
          </div>
        )}

        <div className="content-body">
          {filteredEvents.length === 0 ? (
            <div className="empty-state">
              <p className="empty-text">
                {activeCategory === "all"
                  ? "Your clipboard history is empty."
                  : activeCategory === "text"
                  ? "No text items yet."
                  : "No images yet."}
              </p>
              <p className="empty-hint">Copy some {activeCategory === "all" ? "text or images" : activeCategory} to get started!</p>
            </div>
          ) : (
            <div className="clipboard-list">
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
