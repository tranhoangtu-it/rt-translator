import { useState, useEffect, type ReactNode } from "react";
import { Languages, Moon, Sun } from "lucide-react";
import { useAppStore } from "@/stores/app-store";
import { Button } from "@/components/ui/button";

interface AppLayoutProps {
  children: ReactNode;
}

export function AppLayout({ children }: AppLayoutProps) {
  const isTranscribing = useAppStore((s) => s.stt.isTranscribing);
  const [isDark, setIsDark] = useState(false);

  useEffect(() => {
    setIsDark(document.documentElement.classList.contains("dark"));
  }, []);

  const toggleDarkMode = () => {
    const newMode = !isDark;
    setIsDark(newMode);
    if (newMode) {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
    }
  };

  return (
    <div className="flex h-screen flex-col bg-background text-foreground">
      <header className="sticky top-0 z-50 flex h-12 items-center justify-between border-b bg-background/80 px-4 backdrop-blur-sm">
        <div className="flex items-center gap-2">
          <Languages className="h-5 w-5 text-primary" />
          <h1 className="text-lg font-semibold">RT Translator</h1>
          {isTranscribing && (
            <div className="flex items-center gap-1.5">
              <div className="h-2 w-2 animate-pulse rounded-full bg-red-500" />
              <span className="text-xs text-muted-foreground">Recording</span>
            </div>
          )}
        </div>
        <Button
          variant="ghost"
          size="icon"
          onClick={toggleDarkMode}
          className="cursor-pointer"
        >
          {isDark ? (
            <Sun className="h-4 w-4" />
          ) : (
            <Moon className="h-4 w-4" />
          )}
        </Button>
      </header>
      <main className="flex-1 overflow-auto p-4">{children}</main>
    </div>
  );
}
