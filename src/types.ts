export type ContentType = "Text" | "Image";

export interface ClipboardEntryData {
  content_type: ContentType;
  text_content: string | null;
  image_path: string | null;
  created_at: string;
}

export class ClipboardEntry {
  content_type: ContentType;
  text_content: string | null;
  image_path: string | null;
  created_at: string;

  constructor(data: ClipboardEntryData) {
    this.content_type = data.content_type;
    this.text_content = data.text_content;
    this.image_path = data.image_path;
    this.created_at = data.created_at;
  }

  get type(): string {
    return this.content_type;
  }

  get text(): string {
    return this.text_content || "";
  }

  get image(): string {
    return this.image_path || "";
  }

  get timestamp(): number {
    const date = new Date(this.created_at);
    return Math.floor(date.getTime() / 1000);
  }

  isText(): boolean {
    return this.content_type === "Text";
  }

  isImage(): boolean {
    return this.content_type === "Image";
  }
}
