import { ClipboardEntry } from "../types";
import { FileText, Image as ImageIcon, Copy, Trash2, Clock } from "lucide-react";
import { Card, CardContent, CardFooter, CardHeader } from "./ui/card";
import { Button } from "./ui/button";
import { Badge } from "./ui/badge";

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

  const isText = item.isText();
  const isImage = item.isImage();

  return (
    <Card className="group transition-all hover:shadow-md">
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <Badge variant={isText ? "default" : "secondary"} className="gap-1.5">
            {isText ? (
              <>
                <FileText className="h-3 w-3" />
                <span>Text</span>
              </>
            ) : (
              <>
                <ImageIcon className="h-3 w-3" />
                <span>Image</span>
              </>
            )}
          </Badge>
          <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
            <Clock className="h-3 w-3" />
            <span>{formatTimestamp(item.timestamp)}</span>
          </div>
        </div>
      </CardHeader>

      <CardContent className="pb-3">
        {isText ? (
          <div className="rounded-md bg-muted/50 p-3">
            <p className="max-h-[200px] overflow-y-auto whitespace-pre-wrap break-words text-sm">
              {item.text}
            </p>
          </div>
        ) : isImage ? (
          <div className="flex justify-center rounded-md bg-muted/50 p-3">
            <img
              src={item.image}
              alt="clipboard"
              className="max-h-[300px] rounded-md object-contain"
            />
          </div>
        ) : null}
      </CardContent>

      <CardFooter className="gap-2 pt-0">
        <Button
          variant="outline"
          size="sm"
          className="flex-1 gap-2"
          onClick={() => onPaste(item)}
          title="Paste to clipboard"
        >
          <Copy className="h-4 w-4" />
          <span>Paste</span>
        </Button>
        <Button
          variant="outline"
          size="sm"
          className="gap-2 hover:bg-destructive hover:text-destructive-foreground"
          onClick={() => onDelete(item)}
          title="Delete"
        >
          <Trash2 className="h-4 w-4" />
          <span>Delete</span>
        </Button>
      </CardFooter>
    </Card>
  );
}
