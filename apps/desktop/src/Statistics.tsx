import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
  AreaChart,
  Area,
} from "recharts";
import {
  TrendingUp,
  Clock,
  BarChart3,
  Award,
  MousePointer2,
} from "lucide-react";
import { formatDuration } from "@focus-os/analytics";

interface TopApp {
  name: string;
  seconds: number;
}

interface DailyTotal {
  day: string;
  seconds: number;
}

const daysOptions = [
  { label: "7 Days", value: 7 },
  { label: "14 Days", value: 14 },
  { label: "30 Days", value: 30 },
];

export default function Statistics() {
  const [range, setRange] = useState(7);
  const [totalTime, setTotalTime] = useState(0);
  const [avgSession, setAvgSession] = useState(0);
  const [topApps, setTopApps] = useState<TopApp[]>([]);
  const [dailyTotals, setDailyTotals] = useState<DailyTotal[]>([]);
  const [idlePct, setIdlePct] = useState(0);
  const [mostOpened, setMostOpened] = useState<string | null>(null);

  useEffect(() => {
    const fetch = async () => {
      try {
        const [apps, daily, idle, opened] = await Promise.all([
          invoke<[string, number, number][]>("get_top_apps", { days: range }),
          invoke<[string, number][]>("get_daily_totals", { days: range }),
          invoke<number>("get_idle_percentage", { days: range }),
          invoke<[string, number] | null>("get_most_opened_app", { days: range }),
        ]);
        setTopApps(
          apps
            .slice(0, 10)
            .map(([n, s]) => ({ name: n.replace(".exe", ""), seconds: s }))
        );
        setDailyTotals(
          daily.map(([day, secs]) => ({
            day: new Date(day + "T00:00:00").toLocaleDateString([], {
              month: "short",
              day: "numeric",
            }),
            seconds: secs,
          }))
        );
        setIdlePct(idle);
        setMostOpened(opened?.[0] ?? null);

        // Total time from daily totals
        const total = daily.reduce((acc, [, secs]) => acc + secs, 0);
        setTotalTime(total);
        if (range === 1) {
          const [, todayAvg] = await invoke<[number, number]>("get_today_stats");
          setAvgSession(todayAvg);
        } else if (daily.length > 0) {
          const totalSecs = daily.reduce((acc, [, secs]) => acc + secs, 0);
          const sessionCount = apps.reduce((acc, [, , count]) => acc + count, 0);
          setAvgSession(sessionCount > 0 ? Math.round(totalSecs / sessionCount) : 0);
        }
      } catch (err) {
        console.error("Statistics fetch error:", err);
      }
    };
    fetch();
  }, [range]);

  return (
    <div className="space-y-5">
      {/* Range selector */}
      <Tabs value={String(range)} onValueChange={(v) => setRange(Number(v))}>
        <TabsList className="w-full mb-4 bg-muted/50 p-1">
          {daysOptions.map((opt) => (
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

      {/* Summary cards */}
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
        <Card size="sm">
          <CardHeader className="pb-1.5">
            <CardTitle className="text-[11px] font-medium text-muted-foreground flex items-center gap-1.5">
              <Clock className="size-3" />
              Total Time
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
            <p className="text-xl font-bold tabular-nums">
              {avgSession > 0 ? formatDuration(avgSession) : "—"}
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
              <Award className="size-3" />
              Most Used
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-xl font-bold tabular-nums truncate">
              {mostOpened?.replace(".exe", "") ?? "—"}
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Daily trend chart */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-[11px] font-medium text-muted-foreground uppercase tracking-wider flex items-center gap-1.5">
            <BarChart3 className="size-3" />
            Daily Screen Time
          </CardTitle>
        </CardHeader>
        <CardContent>
          {dailyTotals.length === 0 ? (
            <p className="text-sm text-muted-foreground text-center py-8">
              No data for this period
            </p>
          ) : (
            <div className="h-48">
              <ResponsiveContainer width="100%" height="100%">
                <AreaChart data={dailyTotals} margin={{ top: 5, right: 5, bottom: 5, left: -20 }}>
                  <defs>
                    <linearGradient id="colorSeconds" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%" stopColor="var(--color-primary)" stopOpacity={0.3} />
                      <stop offset="95%" stopColor="var(--color-primary)" stopOpacity={0} />
                    </linearGradient>
                  </defs>
                  <XAxis
                    dataKey="day"
                    tick={{ fontSize: 10, fill: "var(--color-muted-foreground)" }}
                    axisLine={false}
                    tickLine={false}
                  />
                  <YAxis
                    tick={{ fontSize: 10, fill: "var(--color-muted-foreground)" }}
                    axisLine={false}
                    tickLine={false}
                    tickFormatter={(v: number) => {
                      if (v >= 3600) return `${Math.round(v / 3600)}h`;
                      if (v >= 60) return `${Math.round(v / 60)}m`;
                      return `${v}s`;
                    }}
                  />
                  <Tooltip
                    contentStyle={{
                      background: "var(--color-card)",
                      border: "1px solid var(--color-border)",
                      borderRadius: "8px",
                      fontSize: "12px",
                    }}
                    formatter={(value: number) => [formatDuration(value), "Screen time"]}
                  />
                  <Area
                    type="monotone"
                    dataKey="seconds"
                    stroke="var(--color-primary)"
                    strokeWidth={2}
                    fill="url(#colorSeconds)"
                  />
                </AreaChart>
              </ResponsiveContainer>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Top Apps bar chart */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-[11px] font-medium text-muted-foreground uppercase tracking-wider">
            Top Applications
          </CardTitle>
        </CardHeader>
        <CardContent>
          {topApps.length === 0 ? (
            <p className="text-sm text-muted-foreground text-center py-8">
              No data for this period
            </p>
          ) : (
            <div className="h-64">
              <ResponsiveContainer width="100%" height="100%">
                <BarChart
                  data={topApps}
                  layout="vertical"
                  margin={{ top: 5, right: 20, bottom: 5, left: 0 }}
                >
                  <XAxis
                    type="number"
                    tick={{ fontSize: 10, fill: "var(--color-muted-foreground)" }}
                    axisLine={false}
                    tickLine={false}
                    tickFormatter={(v: number) => {
                      if (v >= 3600) return `${Math.round(v / 3600)}h`;
                      if (v >= 60) return `${Math.round(v / 60)}m`;
                      return `${v}s`;
                    }}
                  />
                  <YAxis
                    type="category"
                    dataKey="name"
                    tick={{ fontSize: 11, fill: "var(--color-foreground)" }}
                    axisLine={false}
                    tickLine={false}
                    width={100}
                  />
                  <Tooltip
                    contentStyle={{
                      background: "var(--color-card)",
                      border: "1px solid var(--color-border)",
                      borderRadius: "8px",
                      fontSize: "12px",
                    }}
                    formatter={(value: number) => [formatDuration(value), "Total time"]}
                  />
                  <Bar
                    dataKey="seconds"
                    fill="var(--color-primary)"
                    radius={[0, 4, 4, 0]}
                    barSize={16}
                  />
                </BarChart>
              </ResponsiveContainer>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
