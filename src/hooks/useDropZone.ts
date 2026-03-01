import { useCallback, useEffect, useState } from "react";
import { YOUTUBE_URL_REGEX } from "@/lib/constants";
import { useTaskStore } from "@/stores/useTaskStore";

export function useDropZone() {
  const [isDragging, setIsDragging] = useState(false);
  const addTask = useTaskStore((s) => s.addTask);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback(() => {
    setIsDragging(false);
  }, []);

  const handleDrop = useCallback(
    async (e: React.DragEvent) => {
      e.preventDefault();
      setIsDragging(false);

      // Check for files
      if (e.dataTransfer.files.length > 0) {
        for (const file of Array.from(e.dataTransfer.files)) {
          const ext = file.name.split(".").pop()?.toLowerCase();
          if (ext === "srt" || ext === "vtt" || ext === "txt") {
            await addTask(undefined, file.name);
          }
        }
        return;
      }

      // Check for text (URL)
      const text = e.dataTransfer.getData("text/plain");
      if (text && YOUTUBE_URL_REGEX.test(text)) {
        await addTask(text);
      }
    },
    [addTask],
  );

  // Listen for paste events
  useEffect(() => {
    const handlePaste = async (e: ClipboardEvent) => {
      const text = e.clipboardData?.getData("text/plain");
      if (text) {
        const urls = text
          .split("\n")
          .map((l) => l.trim())
          .filter((l) => YOUTUBE_URL_REGEX.test(l));
        for (const url of urls) {
          await addTask(url);
        }
      }
    };

    document.addEventListener("paste", handlePaste);
    return () => document.removeEventListener("paste", handlePaste);
  }, [addTask]);

  return { isDragging, handleDragOver, handleDragLeave, handleDrop };
}
