export interface ActivitySample {
  exe_name: string;
  window_title: string | null;
  pid: number;
  timestamp: number;
}

export interface Session {
  id: number;
  app_id: number;
  app_name: string;
  window_title: string | null;
  started_at: number;
  ended_at: number | null;
  duration_seconds: number | null;
  idle_seconds: number;
  productive_seconds: number | null;
}

export interface AppInfo {
  id: number;
  name: string;
  exe_name: string;
  icon_path: string | null;
  category: string | null;
}

export interface SystemEvent {
  id: number;
  event_type: string;
  timestamp: number;
  details: string | null;
}

export interface IdleEvent {
  id: number;
  started_at: number;
  ended_at: number | null;
  duration: number | null;
}

export interface DailyStats {
  total_screen_time: number;
  total_idle_time: number;
  productive_time: number;
  top_apps: AppUsage[];
  timeline: Session[];
}

export interface AppUsage {
  app_id: number;
  app_name: string;
  total_duration: number;
  session_count: number;
}
