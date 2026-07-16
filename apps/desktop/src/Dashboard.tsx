import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import {
  Monitor,
  MousePointer2,
  Timer,
  TrendingUp,
  Activity,
} from "lucide-react";
import { formatDuration, formatTime, cleanExeName } from "@focus-os/analytics";

interface TopApp {
  name: string;
  seconds: number;
  sessions: number;
}

interface CurrentSession {
  app_name: string | null;
  window_title: string | null;
  started_at: number | null;
  duration_seconds: number;
}

export default function Dashboard() {
  const [totalTime, setTotalTime] = useState(0);
  const [avgSession, setAvgSession] = useState(0);
  const [topApps, setTopApps] = useState<TopApp[]>([]);
  const [idlePct, setIdlePct] = useState(0);
  const [longest, setLongest] = useState<string | null>(null);
  const [currentSession, setCurrentSession] = useState<CurrentSession | null>(null);

  useEffect(() => {
    const fetch = async () => {
      try {
        const [stats, apps, idle, long, session] = await Promise.all([
          invoke<[number, number]>("get_today_stats"),
          invoke<[string, number, number][]>("get_top_apps", { days: 1 }),
          invoke<number>("get_idle_percentage", { days: 1 }),
          invoke<[string, number] | null>("get_longest_session"),
          invoke<CurrentSession>("get_current_session"),
        ]);
        setTotalTime(stats[0]);
        setAvgSession(stats[1]);
        setTopApps(apps.map(([n, s, c]) => ({ name: n, seconds: s, sessions: c })));
        setIdlePct(idle);
        setLongest(long?.[0] ?? null);
        setCurrentSession(session);
      } catch (err) {
        console.error("Dashboard fetch error:", err);
      }
    };
    fetch();
    const id = setInterval(fetch, 3000);
    return () => clearInterval(id);
  }, []);

  const maxAppSeconds = topApps.length > 0 ? Math.max(...topApps.map((a) => a.seconds)) : 1;

  return (
    <div className="space-y-5">
      {/* Current Session — prominent card */}
      <Card className="overflow-hidden">
        <div className="h-1 bg-gradient-to-r from-primary/40 via-primary/60 to-primary/40" />
        <CardContent className="pt-5 pb-4">
          <div className="flex items-center gap-3 mb-1">
            <div className="size-10 rounded-xl bg-primary/10 flex items-center justify-center">
              <Monitor className="size-5 text-primary" />
            </div>
            <div className="min-w-0 flex-1">
              <p className="text-[11px] font-medium text-muted-foreground uppercase tracking-wider">
                Current Session
              </p>
              {currentSession?.app_name ? (
                <>
                  <p className="text-base font-semibold truncate">
                    {cleanExeName(currentSession.app_name)}
                  </p>
                  {currentSession.window_title && (
                    <p className="text-xs text-muted-foreground truncate">
                      {currentSession.window_title}
                    </p>
                  )}
                </>
              ) : (
                <p className="text-sm text-muted-foreground">No active session</p>
              )}
            </div>
            {currentSession?.app_name && (
              <div className="text-right shrink-0">
                <p className="text-2xl font-bold tabular-nums tracking-tight">
                  {formatDuration(currentSession.duration_seconds)}
                </p>
                <p className="text-[11px] text-muted-foreground">
                  {currentSession.started_at
                    ? `since ${formatTime(currentSession.started_at)}`
                    : ""}
                </p>
              </div>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Stats grid */}
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
        <Card size="sm">
          <CardHeader className="pb-1.5">
            <CardTitle className="text-[11px] font-medium text-muted-foreground flex items-center gap-1.5">
              <Timer className="size-3" />
              Today
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-xl font-bold tabular-nums">{formatDuration(totalTime)}</p>
          </CardContent>
        </Card>
        <Card size="sm">
          <CardHeader className="pb-1.5">
            <CardTitle className="text-[11px] font-medium text-muted-foreground flex items-center gap-1.5">
              <TrendingUp className="size-3" />
              Avg Session
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-xl font-bold tabular-nums">                      {avgSession > 0 ? formatDuration(avgSession as number) : "—"}
            </p>
          </CardContent>
        </Card>
        <Card size="sm">
          <CardHeader className="pb-1.5">
            <CardTitle className="text-[11px] font-medium text-muted-foreground flex items-center gap-1.5">
              <MousePointer2 className="size-3" />
              Idle
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-xl font-bold tabular-nums">
              {(idlePct * 100).toFixed(1)}%
            </p>
          </CardContent>
        </Card>
        <Card size="sm">
          <CardHeader className="pb-1.5">
            <CardTitle className="text-[11px] font-medium text-muted-foreground flex items-center gap-1.5">
              <Activity className="size-3" />
              Longest
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-xl font-bold tabular-nums truncate">
              {longest ?? "—"}
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Idle progress bar */}
      <Card size="sm">
        <CardHeader className="pb-2">
          <CardTitle className="text-[11px] font-medium text-muted-foreground">
            Activity Level
          </CardTitle>
        </CardHeader>
        <CardContent>
          <Progress value={(1 - idlePct) * 100} className="h-2" />
          <div className="flex justify-between mt-1.5">
            <span className="text-[11px] text-muted-foreground">
              {((1 - idlePct) * 100).toFixed(0)}% active
            </span>
            <span className="text-[11px] text-muted-foreground">
              {(idlePct * 100).toFixed(0)}% idle
            </span>
          </div>
        </CardContent>
      </Card>

      {/* Top Apps */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-[11px] font-medium text-muted-foreground uppercase tracking-wider">
            Top Apps Today
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          {topApps.length === 0 && (
            <p className="text-sm text-muted-foreground text-center py-4">
              No activity data yet
            </p>
          )}
          {topApps.slice(0, 8).map((app, i) => (
            <div key={app.name} className="space-y-1">
              <div className="flex justify-between items-center text-sm">
                <div className="flex items-center gap-2 min-w-0 flex-1">
                  <span className="text-[11px] text-muted-foreground font-mono w-4 shrink-0">
                    {i + 1}
                  </span>
                  <span className="truncate font-medium">
                    {cleanExeName(app.name)}
                  </span>
                </div>
                <div className="flex items-center gap-3 shrink-0">
                  <span className="text-xs text-muted-foreground tabular-nums">
                    {app.sessions} session{app.sessions !== 1 ? "s" : ""}
                  </span>
                  <span className="text-sm font-semibold tabular-nums w-16 text-right">
                    {formatDuration(app.seconds)}
                  </span>
                </div>
              </div>
              <div className="h-1.5 rounded-full bg-muted overflow-hidden">
                <div
                  className="h-full rounded-full bg-primary/70 transition-all duration-500"
                  style={{ width: `${(app.seconds / maxAppSeconds) * 100}%` }}
                />
              </div>
            </div>
          ))}
        </CardContent>
      </Card>
    </div>
  );
}
