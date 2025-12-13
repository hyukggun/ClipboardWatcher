import { ClipboardEntry } from "../types";

interface ClipboardCardProps {
  item: ClipboardEntry;
  onDelete: (item: ClipboardEntry) => void;
  onPaste: (item: ClipboardEntry) => void;
}

export default function ClipboardCard({ item, onDelete, onPaste }: ClipboardCardProps) {
  const formatTimestamp = (timestamp: number) => {
    const date = new Date(timestamp * 1000);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);

    if (minutes < 1) return "Just now";
    if (minutes < 60) return `${minutes}분 전`;
    if (hours < 24) return `${hours}시간 전`;
    if (days < 7) return `${days}일 전`;
    return date.toLocaleDateString();
  };

  return (
    <div className="clipboard-card">
      <div className="card-header">
        <div className="card-type">
          {item.isText() ? (
            <>
              <span className="type-icon">📝</span>
              <span className="type-label">Text</span>
            </>
          ) : (
            <>
              <span className="type-icon">🖼️</span>
              <span className="type-label">Image</span>
            </>
          )}
        </div>
        <div className="card-time">{formatTimestamp(item.timestamp)}</div>
      </div>

      <div className="card-content">
        {item.isText() ? (
          <div className="text-content">{item.text}</div>
        ) : item.isImage() ? (
          <div className="image-content">
            <img src={item.image} alt="clipboard" className="clipboard-image" />
          </div>
        ) : null}
      </div>

      <div className="card-actions">
        <button
          className="action-btn paste-btn"
          onClick={() => onPaste(item)}
          title="Paste to clipboard"
        >
          <span>📋</span> Paste
        </button>
        <button
          className="action-btn delete-btn"
          onClick={() => onDelete(item)}
          title="Delete"
        >
          <span>🗑️</span> Delete
        </button>
      </div>
    </div>
  );
}
