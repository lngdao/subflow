import { Toaster } from "sonner";
import { AppShell } from "@/components/layout/AppShell";

function App() {
  return (
    <>
      <AppShell />
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
