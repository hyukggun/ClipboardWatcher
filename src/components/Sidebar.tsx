import { useState } from "react";

interface SidebarProps {
  totalCount: number;
  textCount: number;
  imageCount: number;
  onCategoryChange: (category: "all" | "text" | "images") => void;
  onToggle: (expanded: boolean) => void;
}

export default function Sidebar({
  totalCount,
  textCount,
  imageCount,
  onCategoryChange,
  onToggle,
}: SidebarProps) {
  const [expanded, setExpanded] = useState(true);
  const [activeCategory, setActiveCategory] = useState<"all" | "text" | "images">("all");

  const handleToggle = () => {
    const newExpanded = !expanded;
    setExpanded(newExpanded);
    onToggle(newExpanded);
  };

  const handleCategoryClick = (category: "all" | "text" | "images") => {
    console.log("[SIDEBAR] Category clicked:", category);
    setActiveCategory(category);
    onCategoryChange(category);
    console.log("[SIDEBAR] Category changed to:", category);
  };

  return (
    <aside className={`sidebar ${expanded ? "" : "collapsed"}`}>
      <div className="sidebar-header">
        {expanded && <h1 className="app-title">Clipboard Watcher</h1>}
        <button
          className={`toggle-button ${expanded ? "" : "collapsed"}`}
          onClick={handleToggle}
          title={expanded ? "Collapse sidebar" : "Expand sidebar"}
        >
          {expanded ? "‹" : "›"}
        </button>
      </div>

      <nav className="sidebar-nav">
        <div className="nav-section">
          {expanded && <h3 className="section-title">Categories</h3>}
          <ul className="category-list">
            <li
              className={`category-item ${activeCategory === "all" ? "active" : ""}`}
              onClick={() => handleCategoryClick("all")}
              title="All"
            >
              <span className="category-icon">📋</span>
              {expanded && (
                <>
                  <span className="category-name">All</span>
                  <span className="category-count">{totalCount}</span>
                </>
              )}
            </li>
            <li
              className={`category-item ${activeCategory === "text" ? "active" : ""}`}
              onClick={() => handleCategoryClick("text")}
              title="Text"
            >
              <span className="category-icon">📝</span>
              {expanded && (
                <>
                  <span className="category-name">Text</span>
                  <span className="category-count">{textCount}</span>
                </>
              )}
            </li>
            <li
              className={`category-item ${activeCategory === "images" ? "active" : ""}`}
              onClick={() => handleCategoryClick("images")}
              title="Images"
            >
              <span className="category-icon">🖼️</span>
              {expanded && (
                <>
                  <span className="category-name">Images</span>
                  <span className="category-count">{imageCount}</span>
                </>
              )}
            </li>
          </ul>
        </div>
      </nav>

      <div className="sidebar-footer">
        <button className="settings-button" title="Preferences">
          <span className="settings-icon">⚙️</span>
          {expanded && <span>Preferences</span>}
        </button>
      </div>
    </aside>
  );
}
