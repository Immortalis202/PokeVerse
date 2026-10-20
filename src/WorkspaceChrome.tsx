import {
  BookOpen,
  Users,
  Radio,
  Swords,
  History,
  Settings,
  Leaf,
  Shield,
} from "lucide-react";
import type { Campaign } from "./bindings/Campaign";

const pages = [
  { name: "Library", icon: Users },
  { name: "Compendium", icon: BookOpen },
  { name: "Room", icon: Radio },
  { name: "Battle", icon: Swords },
  { name: "Training encounters", icon: Leaf },
  { name: "History", icon: History },
  { name: "Settings", icon: Settings },
];

export function WorkspaceRail({
  page,
  navigate,
  connected,
  desktop,
}: {
  page: string;
  navigate: (page: string) => void;
  connected: boolean;
  desktop: boolean;
}) {
  const item = ({ name, icon: Icon }: (typeof pages)[number]) => (
    <button
      type="button"
      key={name}
      className={`rail-button ${page === name ? "selected" : ""}`}
      aria-label={name}
      aria-current={page === name ? "page" : undefined}
      onClick={() => navigate(name)}
    >
      <Icon size={20} aria-hidden="true" />
      {name === "Room" && connected && (
        <span className="dot online" aria-hidden="true" />
      )}
      <span className="rail-tooltip" aria-hidden="true">
        {name}
      </span>
    </button>
  );
  return (
    <aside className="sidebar">
      <button
        type="button"
        className="brand-mark rail-button"
        aria-label="PokeVerse — Library"
        onClick={() => navigate("Library")}
      >
        P
        <span className="rail-tooltip" aria-hidden="true">
          PokeVerse · v0.20.0
        </span>
      </button>
      <nav aria-label="Workspace">
        {pages.filter((p) => p.name !== "Settings").map(item)}
      </nav>
      <div className="sidebar-bottom">
        <div
          className="rail-local"
          role="img"
          aria-label={
            desktop ? "Local library available offline" : "Browser preview"
          }
        >
          <Shield size={18} aria-hidden="true" />
        </div>
        <nav aria-label="Preferences">{item(pages[pages.length - 1]!)}</nav>
      </div>
    </aside>
  );
}

export function WorkspaceToolbar({
  page,
  campaign,
  playerId,
  connected,
  avatar,
}: {
  page: string;
  campaign: Campaign | null;
  playerId: string;
  connected: boolean;
  avatar: string;
}) {
  return (
    <header className="topbar">
      <div className="workspace-heading">
        <strong className="toolbar-brand">PokeVerse</strong>
        <span className="toolbar-divider" />
        <span className="toolbar-page">{page}</span>
        {campaign && (
          <span className="toolbar-campaign" title={campaign.name}>
            {campaign.name}
          </span>
        )}
      </div>
      <div className="toolbar-status">
        {campaign && (
          <div className="toolbar-roles" aria-label="Your room roles">
            {campaign.host_id === playerId && (
              <span className="role-badge">Host</span>
            )}
            {campaign.gm_id === playerId && (
              <span className="role-badge">GM</span>
            )}
            {campaign.host_id !== playerId && campaign.gm_id !== playerId && (
              <span className="role-badge">Player</span>
            )}
          </div>
        )}
        <span
          className={`connection-status ${campaign ? (connected ? "connected" : "paused") : "local"}`}
          role="status"
        >
          <span
            className={`dot ${connected || !campaign ? "online" : ""}`}
            aria-hidden="true"
          />
          {campaign
            ? connected
              ? "Connected"
              : "Host unavailable"
            : "Offline ready"}
        </span>
        <div className="avatar" aria-hidden="true">
          {avatar}
        </div>
      </div>
    </header>
  );
}
