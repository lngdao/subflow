import { useCallback, useState } from "react";

export function useDropZone() {
  const [isDragging, setIsDragging] = useState(false);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback(() => {
    setIsDragging(false);
  }, []);

  const extractPaths = useCallback((e: React.DragEvent): string[] => {
    e.preventDefault();
    setIsDragging(false);

    const paths: string[] = [];
    if (e.dataTransfer.files.length > 0) {
      for (const file of Array.from(e.dataTransfer.files)) {
        const ext = file.name.split(".").pop()?.toLowerCase();
        if (ext === "srt" || ext === "vtt" || ext === "txt") {
          // In Tauri, File objects have a `path` property
          const filePath = (file as File & { path?: string }).path || file.name;
          paths.push(filePath);
        }
      }
    }
    return paths;
  }, []);

  return { isDragging, handleDragOver, handleDragLeave, extractPaths };
}
