import { useState } from "react";
import { Layers, FileText, Image, Settings, ChevronLeft, ChevronRight } from "lucide-react";
import { Badge } from "./ui/badge";
import { Button } from "./ui/button";
import { cn } from "../lib/utils";

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
  };

  const categories = [
    { id: "all" as const, icon: Layers, label: "All", count: totalCount },
    { id: "text" as const, icon: FileText, label: "Text", count: textCount },
    { id: "images" as const, icon: Image, label: "Images", count: imageCount },
  ];

  return (
    <aside
      className={cn(
        "flex flex-col border-r bg-muted/20 transition-all duration-200",
        expanded ? "w-60" : "w-16"
      )}
    >
      {/* Header */}
      <div className="flex items-center justify-between border-b px-3 py-3">
        {expanded && (
          <h1 className="text-sm font-semibold tracking-tight">
            Clipboard Watcher
          </h1>
        )}
        <Button
          variant="ghost"
          size="icon"
          onClick={handleToggle}
          className={cn("h-8 w-8", !expanded && "mx-auto")}
          title={expanded ? "Collapse sidebar" : "Expand sidebar"}
        >
          {expanded ? (
            <ChevronLeft className="h-4 w-4" />
          ) : (
            <ChevronRight className="h-4 w-4" />
          )}
        </Button>
      </div>

      {/* Categories */}
      <nav className="flex-1 space-y-1 p-2">
        {expanded && (
          <p className="mb-2 px-2 text-xs font-medium text-muted-foreground">
            CATEGORIES
          </p>
        )}
        <ul className="space-y-1">
          {categories.map((category) => {
            const Icon = category.icon;
            const isActive = activeCategory === category.id;

            return (
              <li key={category.id}>
                <button
                  onClick={() => handleCategoryClick(category.id)}
                  className={cn(
                    "flex w-full items-center gap-3 rounded-lg px-3 py-2 text-sm transition-colors",
                    "hover:bg-accent hover:text-accent-foreground",
                    isActive && "bg-accent text-accent-foreground font-medium",
                    !expanded && "justify-center"
                  )}
                  title={category.label}
                >
                  <Icon className={cn("h-5 w-5 shrink-0", isActive && "text-primary")} />
                  {expanded && (
                    <>
                      <span className="flex-1 text-left">{category.label}</span>
                      <Badge
                        variant="secondary"
                        className="h-5 min-w-5 px-1.5 text-xs"
                      >
                        {category.count}
                      </Badge>
                    </>
                  )}
                </button>
              </li>
            );
          })}
        </ul>
      </nav>

      {/* Footer */}
      <div className="border-t p-2">
        <Button
          variant="ghost"
          className={cn(
            "w-full justify-start gap-3",
            !expanded && "justify-center"
          )}
          title="Preferences"
        >
          <Settings className="h-5 w-5 shrink-0" />
          {expanded && <span>Preferences</span>}
        </Button>
      </div>
    </aside>
  );
}
