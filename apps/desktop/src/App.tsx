import { useState } from "react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Activity, BarChart3, Clock, Grid3X3 } from "lucide-react";
import Dashboard from "./Dashboard";
import Timeline from "./Timeline";
import Statistics from "./Statistics";
import Applications from "./Applications";

export default function App() {
  const [tab, setTab] = useState("dashboard");

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="sticky top-0 z-10 border-b border-border/50 bg-background/80 backdrop-blur-sm">
        <div className="max-w-4xl mx-auto px-4 py-3 flex items-center justify-between">
          <div className="flex items-center gap-2.5">
            <div className="size-7 rounded-lg bg-primary/10 flex items-center justify-center">
              <Activity className="size-4 text-primary" />
            </div>
            <h1 className="text-sm font-semibold tracking-tight">FocusOS</h1>
          </div>
          <span className="text-[11px] text-muted-foreground font-mono">
            privacy-first · local-only
          </span>
        </div>
      </header>

      {/* Main content */}
      <main className="max-w-4xl mx-auto px-4 py-5">
        <Tabs value={tab} onValueChange={setTab}>
          <TabsList className="w-full mb-6 bg-muted/50 p-1 h-auto">
            <TabsTrigger value="dashboard" className="flex-1 py-2 data-active:bg-background">
              <Grid3X3 className="size-3.5 mr-1.5" />
              Dashboard
            </TabsTrigger>
            <TabsTrigger value="timeline" className="flex-1 py-2 data-active:bg-background">
              <Clock className="size-3.5 mr-1.5" />
              Timeline
            </TabsTrigger>
            <TabsTrigger value="statistics" className="flex-1 py-2 data-active:bg-background">
              <BarChart3 className="size-3.5 mr-1.5" />
              Statistics
            </TabsTrigger>
            <TabsTrigger value="applications" className="flex-1 py-2 data-active:bg-background">
              <Grid3X3 className="size-3.5 mr-1.5" />
              Apps
            </TabsTrigger>
          </TabsList>

          <TabsContent value="dashboard">
            <Dashboard />
          </TabsContent>
          <TabsContent value="timeline">
            <Timeline />
          </TabsContent>
          <TabsContent value="statistics">
            <Statistics />
          </TabsContent>
          <TabsContent value="applications">
            <Applications />
          </TabsContent>
        </Tabs>
      </main>
    </div>
  );
}
