import { useTranslation } from "react-i18next";
import { Toaster } from "sonner";
import { AppShell } from "@/components/layout/AppShell";
import { useUiStore } from "@/stores/useUiStore";

function SplashScreen() {
  const { t } = useTranslation();
  const appReady = useUiStore((s) => s.appReady);

  return (
    <div
      className={`fixed inset-0 z-[100] flex flex-col items-center justify-center bg-background transition-opacity duration-500 ${
        appReady ? "opacity-0 pointer-events-none" : "opacity-100"
      }`}
    >
      <h1 className="text-2xl font-bold text-foreground tracking-tight">
        {t("app.name")}
      </h1>
      <p className="text-sm text-muted-foreground mt-1">
        {t("app.subtitle")}
      </p>
    </div>
  );
}

function App() {
  const splashVisible = useUiStore((s) => s.splashVisible);

  return (
    <>
      <AppShell />
      {splashVisible && <SplashScreen />}
      <Toaster
        position="bottom-center"
        theme="dark"
        richColors
        toastOptions={{
          style: {
            background: "var(--card)",
            color: "var(--foreground)",
            border: "1px solid var(--border)",
          },
        }}
      />
    </>
  );
}

export default App;
