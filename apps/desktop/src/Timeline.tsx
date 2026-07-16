import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Card, CardContent } from "@/components/ui/card";
import { Clock, ChevronRight } from "lucide-react";
import { formatDuration, formatTime, cleanExeName } from "@focus-os/analytics";

interface Session {
  app: string;
  title: string;
  start: number;
  end: number;
  duration: number;
}

// No local formatters needed — using @focus-os/analytics

// Generate a deterministic color from app name
function appColor(app: string): string {
  const colors = [
    "bg-blue-500/80",
    "bg-emerald-500/80",
    "bg-violet-500/80",
    "bg-amber-500/80",
    "bg-rose-500/80",
    "bg-cyan-500/80",
    "bg-pink-500/80",
    "bg-lime-500/80",
    "bg-indigo-500/80",
    "bg-teal-500/80",
  ];
  let hash = 0;
  for (let i = 0; i < app.length; i++) {
    hash = ((hash << 5) - hash) + app.charCodeAt(i);
    hash |= 0;
  }
  return colors[Math.abs(hash) % colors.length];
}

export default function Timeline() {
  const [sessions, setSessions] = useState<Session[]>([]);

  useEffect(() => {
    const fetch = async () => {
      try {
        const raw = await invoke<[string, string, number, number, number][]>("get_timeline");
        setSessions(
          raw.map(([app, title, start, end, dur]) => ({
            app,
            title,
            start,
            end,
            duration: dur,
          }))
        );
      } catch (err) {
        console.error("Timeline fetch error:", err);
      }
    };
    fetch();
    const id = setInterval(fetch, 5000);
    return () => clearInterval(id);
  }, []);

  // Calculate relative widths based on total time span
  const maxTime = sessions.length > 0
    ? Math.max(...sessions.map((s) => s.end))
    : 0;
  const minTime = sessions.length > 0
    ? Math.min(...sessions.map((s) => s.start))
    : 0;
  const timeSpan = Math.max(maxTime - minTime, 1);

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-2 text-[11px] text-muted-foreground uppercase tracking-wider font-medium">
        <Clock className="size-3.5" />
        Today's Timeline
      </div>

      {sessions.length === 0 && (
        <Card>
          <CardContent className="py-8 text-center">
            <p className="text-sm text-muted-foreground">No sessions recorded today</p>
          </CardContent>
        </Card>
      )}

      <ScrollArea className="h-[calc(100vh-14rem)]">
        <div className="space-y-1.5">
          {sessions.map((s, i) => {
            const leftPct = ((s.start - minTime) / timeSpan) * 100;
            const widthPct = Math.max(((s.end - s.start) / timeSpan) * 100, 0.5);
            return (
              <Card key={i} className="hover:bg-card/80 transition-colors">
                <CardContent className="p-3">
                  <div className="flex items-center gap-3 mb-2">
                    <div
                      className={`size-2.5 rounded-full shrink-0 ${appColor(s.app)}`}
                    />
                    <div className="min-w-0 flex-1">
                      <p className="text-sm font-medium truncate">
                        {cleanExeName(s.app)}
                      </p>
                      {s.title && (
                        <p className="text-[11px] text-muted-foreground truncate">
                          {s.title}
                        </p>
                      )}
                    </div>
                    <div className="text-right shrink-0">
                      <p className="text-xs text-muted-foreground tabular-nums">
                        {formatTime(s.start)}
                      </p>
                      <p className="text-sm font-semibold tabular-nums">
                        {formatDuration(s.duration)}
                      </p>
                    </div>
                  </div>

                  {/* Visual timeline bar */}
                  <div className="h-6 rounded-md bg-muted/50 relative overflow-hidden">
                    <div
                      className={`absolute inset-y-0 rounded-md ${appColor(s.app)} transition-all`}
                      style={{
                        left: `${leftPct}%`,
                        width: `${widthPct}%`,
                        minWidth: "4px",
                      }}
                    />
                    <div className="absolute inset-0 flex items-center px-2">
                      <span className="text-[10px] text-muted-foreground/70 tabular-nums">
                        {formatTime(s.start)}
                      </span>
                      <ChevronRight className="size-2.5 text-muted-foreground/40 mx-1" />
                      <span className="text-[10px] text-muted-foreground/70 tabular-nums">
                        {formatTime(s.end)}
                      </span>
                    </div>
                  </div>
                </CardContent>
              </Card>
            );
          })}
        </div>
      </ScrollArea>
    </div>
  );
}
