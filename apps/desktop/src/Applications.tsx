import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent } from "@/components/ui/card";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Search, ChevronRight, ArrowLeft } from "lucide-react";
import { Button } from "@/components/ui/button";
import { formatDuration, formatTime, formatDate } from "@focus-os/analytics";

interface AppUsage {
  name: string;
  seconds: number;
  sessions: number;
  exe_name: string;
}

interface Session {
  app: string;
  title: string;
  start: number;
  end: number;
  duration: number;
}

const rangeOptions = [
  { label: "Today", value: 1 },
  { label: "7 Days", value: 7 },
  { label: "30 Days", value: 30 },
];

export default function Applications() {
  const [range, setRange] = useState(1);
  const [apps, setApps] = useState<AppUsage[]>([]);
  const [selectedApp, setSelectedApp] = useState<string | null>(null);
  const [appHistory, setAppHistory] = useState<Session[]>([]);
  const [searchQuery, setSearchQuery] = useState("");

  useEffect(() => {
    const fetch = async () => {
      try {
        // Get app usage data using get_top_apps with the selected range
        const raw = await invoke<[string, number, number][]>("get_top_apps", {
          days: range,
        });
        setApps(
          raw.map(([name, secs, count]) => ({
            name: name.replace(".exe", ""),
            exe_name: name,
            seconds: secs,
            sessions: count,
          }))
        );
      } catch (err) {
        console.error("Applications fetch error:", err);
      }
    };
    fetch();
  }, [range]);

  useEffect(() => {
    if (!selectedApp) return;
    const fetch = async () => {
      try {
        const raw = await invoke<[string, string, number, number, number][]>(
          "get_app_history",
          { exeName: selectedApp, days: range }
        );
        setAppHistory(
          raw.map(([app, title, start, end, dur]) => ({
            app,
            title,
            start,
            end,
            duration: dur,
          }))
        );
      } catch (err) {
        console.error("App history fetch error:", err);
      }
    };
    fetch();
  }, [selectedApp, range]);

  const filteredApps = apps.filter(
    (a) =>
      !searchQuery ||
      a.name.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const maxAppSeconds = filteredApps.length > 0
    ? Math.max(...filteredApps.map((a) => a.seconds))
    : 1;

  // Detail view
  if (selectedApp) {
    const appInfo = apps.find((a) => a.exe_name === selectedApp);
    return (
      <div className="space-y-4">
        <Button
          variant="ghost"
          size="sm"
          onClick={() => setSelectedApp(null)}
          className="mb-2 -ml-2"
        >
          <ArrowLeft className="size-3.5 mr-1" />
          Back to Apps
        </Button>

        <Card>
          <CardContent className="pt-5 pb-4">
            <div className="flex items-center justify-between mb-3">
              <div>
                <h2 className="text-lg font-semibold">{appInfo?.name ?? selectedApp}</h2>
                {appInfo && (
                  <p className="text-xs text-muted-foreground">
                    {formatDuration(appInfo.seconds)} across {appInfo.sessions} session
                    {appInfo.sessions !== 1 ? "s" : ""}
                  </p>
                )}
              </div>
            </div>

            {/* Range selector inline */}
            <Tabs
              value={String(range)}
              onValueChange={(v) => setRange(Number(v))}
            >
              <TabsList className="bg-muted/50 p-1">
                {rangeOptions.map((opt) => (
                  <TabsTrigger
                    key={opt.value}
                    value={String(opt.value)}
                    className="text-xs py-1 px-3"
                  >
                    {opt.label}
                  </TabsTrigger>
                ))}
              </TabsList>
            </Tabs>
          </CardContent>
        </Card>

        <ScrollArea className="h-[calc(100vh-20rem)]">
          <div className="space-y-1.5">
            {appHistory.map((s, i) => (
              <Card key={i} className="hover:bg-card/80 transition-colors">
                <CardContent className="p-3 flex items-center justify-between">
                  <div className="min-w-0 flex-1">
                    {s.title && (
                      <p className="text-sm truncate">{s.title}</p>
                    )}
                    <p className="text-[11px] text-muted-foreground">
                      {formatDate(s.start)} · {formatTime(s.start)} – {formatTime(s.end)}
                    </p>
                  </div>
                  <div className="text-right shrink-0 ml-4">
                    <p className="text-sm font-semibold tabular-nums">
                      {formatDuration(s.duration)}
                    </p>
                  </div>
                </CardContent>
              </Card>
            ))}
            {appHistory.length === 0 && (
              <p className="text-sm text-muted-foreground text-center py-8">
                No sessions in this period
              </p>
            )}
          </div>
        </ScrollArea>
      </div>
    );
  }

  // List view
  return (
    <div className="space-y-4">
      <div className="flex items-center gap-3">
        <Tabs
          value={String(range)}
          onValueChange={(v) => setRange(Number(v))}
          className="flex-1"
        >
          <TabsList className="bg-muted/50 p-1 w-full">
            {rangeOptions.map((opt) => (
              <TabsTrigger
                key={opt.value}
                value={String(opt.value)}
                className="flex-1 py-1.5 text-xs"
              >
                {opt.label}
              </TabsTrigger>
            ))}
          </TabsList>
        </Tabs>
      </div>

      {/* Search */}
      <div className="relative">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 size-3.5 text-muted-foreground" />
        <input
          type="text"
          placeholder="Search applications..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="w-full h-9 rounded-lg border border-border bg-background pl-9 pr-3 text-sm outline-none focus:border-ring focus:ring-1 focus:ring-ring/30 transition-colors placeholder:text-muted-foreground/50"
        />
      </div>

      {/* App list */}
      <ScrollArea className="h-[calc(100vh-16rem)]">
        <div className="space-y-1.5">
          {filteredApps.map((app) => (
            <Card
              key={app.exe_name}
              className="hover:bg-card/80 transition-colors cursor-pointer"
              onClick={() => setSelectedApp(app.exe_name)}
            >
              <CardContent className="p-3 flex items-center gap-3">
                <div className="size-9 rounded-lg bg-primary/10 flex items-center justify-center shrink-0">
                  <span className="text-xs font-bold text-primary uppercase tracking-wider">
                    {app.name.charAt(0)}
                  </span>
                </div>
                <div className="min-w-0 flex-1">
                  <p className="text-sm font-medium truncate">{app.name}</p>
                  <p className="text-[11px] text-muted-foreground">
                    {app.sessions} session{app.sessions !== 1 ? "s" : ""}
                  </p>
                </div>
                <div className="text-right shrink-0">
                  <p className="text-sm font-semibold tabular-nums">
                    {formatDuration(app.seconds)}
                  </p>
                </div>
                <ChevronRight className="size-4 text-muted-foreground/40" />
              </CardContent>
              <div className="h-1 bg-muted/50 mx-3 mb-2 rounded-full overflow-hidden">
                <div
                  className="h-full rounded-full bg-primary/60 transition-all"
                  style={{ width: `${(app.seconds / maxAppSeconds) * 100}%` }}
                />
              </div>
            </Card>
          ))}
          {filteredApps.length === 0 && (
            <p className="text-sm text-muted-foreground text-center py-8">
              {searchQuery ? "No matching applications" : "No applications used yet"}
            </p>
          )}
        </div>
      </ScrollArea>
    </div>
  );
}
