import { useCallback, useEffect, useRef, useState } from "react";

const CURRENT_VERSION = __APP_VERSION__;
const GITHUB_API_URL =
  "https://api.github.com/repos/lngdao/subflow/releases/latest";
const CHECK_INTERVAL = 6 * 60 * 60 * 1000; // 6 hours

interface UpdateInfo {
  available: boolean;
  version: string;
  url: string;
  changelog: string;
}

interface UseUpdateCheckerReturn {
  update: UpdateInfo | null;
  checking: boolean;
  checkNow: () => void;
}

function compareSemver(a: string, b: string): number {
  const pa = a.replace(/^v/, "").split(".").map(Number);
  const pb = b.replace(/^v/, "").split(".").map(Number);
  for (let i = 0; i < 3; i++) {
    const diff = (pb[i] || 0) - (pa[i] || 0);
    if (diff !== 0) return diff;
  }
  return 0;
}

export function useUpdateChecker(): UseUpdateCheckerReturn {
  const [update, setUpdate] = useState<UpdateInfo | null>(null);
  const [checking, setChecking] = useState(false);
  const intervalRef = useRef<ReturnType<typeof setInterval>>(undefined);

  const checkForUpdates = useCallback(async () => {
    setChecking(true);
    try {
      const res = await fetch(GITHUB_API_URL);
      if (!res.ok) return;
      const data = await res.json();
      const latestVersion = (data.tag_name as string).replace(/^v/, "");
      if (compareSemver(CURRENT_VERSION, latestVersion) > 0) {
        setUpdate({
          available: true,
          version: latestVersion,
          url: data.html_url,
          changelog: data.body || "",
        });
      } else {
        setUpdate(null);
      }
    } catch {
      // Silently fail - network might be unavailable
    } finally {
      setChecking(false);
    }
  }, []);

  useEffect(() => {
    checkForUpdates();
    intervalRef.current = setInterval(checkForUpdates, CHECK_INTERVAL);
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, [checkForUpdates]);

  return { update, checking, checkNow: checkForUpdates };
}
