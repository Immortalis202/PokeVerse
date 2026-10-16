import TrainingPointPool from "./TrainingPointPool";
import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  Users,
  Radio,
  Swords,
  History,
  Plus,
  ArrowUpRight,
  Copy,
  Download,
  Upload,
  RefreshCw,
  Leaf,
  Check,
  X,
  WifiOff,
  Search,
} from "lucide-react";
import { api, call, desktop, download, errorOf } from "./api";
import type { Catalog } from "./catalog";
import type { Library } from "./bindings/Library";
import type { Sheet } from "./bindings/Sheet";
import type { Campaign } from "./bindings/Campaign";
import type { Action } from "./bindings/Action";
import type { Command } from "./bindings/Command";
import type { AppError } from "./bindings/AppError";
import type { RoomInfo } from "./bindings/RoomInfo";
import type { ServerMessage } from "./bindings/ServerMessage";
import { cloneTrainer, newSheet, sheetValues } from "./sheets";
import SheetEditor from "./SheetEditor";
import CatalogView from "./CatalogView";
import BattleView from "./BattleView";
import GmAssignment from "./GmAssignment";
import WorkflowRoom from "./WorkflowRoom";
import TrainingEncounters from "./TrainingEncounters";
import { WorkspaceRail, WorkspaceToolbar } from "./WorkspaceChrome";
import type { WorkflowRequest } from "./bindings/WorkflowRequest";

export default function App() {
  const [page, setPage] = useState("Library");
  const [catalog, setCatalog] = useState<Catalog | null>(null);
  const [library, setLibrary] = useState<Library>({
    player_id: "browser-preview",
    sheets: [],
    history: [],
  });
  const [campaigns, setCampaigns] = useState<Campaign[]>([]);
  const [campaign, setCampaign] = useState<Campaign | null>(null);
  const [roomInfo, setRoomInfo] = useState<RoomInfo | null>(null);
  const [connected, setConnected] = useState(false);
  const [connectionMessage, setConnectionMessage] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<AppError | null>(null);
  const [toast, setToast] = useState("");
  const [editor, setEditor] = useState<Sheet | null>(null);
  const [drafts, setDrafts] = useState<WorkflowRequest[]>([]);
  const [editingDraft, setEditingDraft] = useState<WorkflowRequest | null>(
    null,
  );
  const [trainerId, setTrainerId] = useState("");
  const [search, setSearch] = useState("");
  const [showArchived, setShowArchived] = useState(false);
  const [campaignName, setCampaignName] = useState("");
  const [campaignId, setCampaignId] = useState("");
  const [port, setPort] = useState(39231);
  const [address, setAddress] = useState("ws://192.168.1.10:39231/room");
  const [token, setToken] = useState("");
  const [selection, setSelection] = useState<string[]>([]);
  const [resolutions, setResolutions] = useState<Record<string, string>>({});
  const [retrySync, setRetrySync] = useState(false);
  const [rewardSheet, setRewardSheet] = useState("");
  const [rewardXP, setRewardXP] = useState(0);
  const [rewardReason, setRewardReason] = useState("");
  const [backups, setBackups] = useState<string[]>([]);
  const [restoreId, setRestoreId] = useState("");
  const [confirmRestore, setConfirmRestore] = useState(false);
  const [pending, setPending] = useState<Command | null>(null);
  const campaignRef = useRef<Campaign | null>(null);
  const importRef = useRef<HTMLInputElement>(null);
  const mainRef = useRef<HTMLElement>(null);
  useEffect(() => {
    mainRef.current?.scrollTo({ top: 0, left: 0 });
  }, [page]);
  const trainers = library.sheets.filter(
    (s) => s.kind === "trainer" && (showArchived || !s.archived),
  );
  const trainer = trainers.find((t) => t.id === trainerId) ?? trainers[0];
  const squad = library.sheets
    .filter(
      (s) =>
        s.kind === "pokemon" &&
        s.trainer_id === trainer?.id &&
        (showArchived || !s.archived) &&
        s.name.toLowerCase().includes(search.toLowerCase()),
    )
    .sort((a, b) => a.squad_order - b.squad_order);
  const setRoom = (next: Campaign) => {
    if (
      !campaignRef.current ||
      next.id !== campaignRef.current.id ||
      next.revision >= campaignRef.current.revision
    ) {
      campaignRef.current = next;
      setCampaign(next);
    }
  };
  const refresh = async () => {
    const [l, cs, ds] = await Promise.all([
      api.library(),
      api.campaigns(),
      api.drafts(),
    ]);
    setLibrary(l);
    setCampaigns(cs);
    setDrafts(ds);
  };
  async function run(work: () => Promise<void>) {
    setBusy(true);
    setError(null);
    try {
      await work();
      return true;
    } catch (e) {
      setError(errorOf(e));
      return false;
    } finally {
      setBusy(false);
    }
  }
  useEffect(() => {
    if (desktop) {
      Promise.all([
        api.catalog(),
        api.library(),
        api.campaigns(),
        call<string[]>("list_backups"),
        api.drafts(),
      ])
        .then(([c, l, cs, bs, ds]) => {
          setCatalog(c);
          setLibrary(l);
          setCampaigns(cs);
          setBackups(bs);
          setDrafts(ds);
        })
        .catch((e) => setError(errorOf(e)));
    } else
      api
        .catalog()
        .then(setCatalog)
        .catch((e) => setError(errorOf(e)));
    const listeners = desktop
      ? [
          listen<ServerMessage>("room-message", (event) => {
            const m = event.payload;
            if ("campaign" in m) setRoom(m.campaign);
            if (m.type === "error" && !m.command_id) setError(m.error);
          }),
          listen<{ connected: boolean; message: string }>(
            "room-status",
            (event) => {
              setConnected(event.payload.connected);
              setConnectionMessage(event.payload.message);
            },
          ),
        ]
      : [];
    return () => {
      listeners.forEach((p) => p.then((unlisten) => unlisten()));
    };
  }, []);
  useEffect(() => {
    if (toast) {
      const t = setTimeout(() => setToast(""), 5000);
      return () => clearTimeout(t);
    }
  }, [toast]);
  const rememberPending = (cmd: Command | null, cid: string) => {
    setPending(cmd);
    if (cmd) localStorage.setItem(`pending:${cid}`, JSON.stringify(cmd));
    else localStorage.removeItem(`pending:${cid}`);
  };
  const loadPending = (cid: string) => {
    try {
      setPending(JSON.parse(localStorage.getItem(`pending:${cid}`) ?? "null"));
    } catch {
      setPending(null);
    }
  };
  async function send(action: Action) {
    const c = campaignRef.current;
    if (!c) throw new Error("Join a room first.");
    if (pending)
      throw new Error(
        "Resolve the unacknowledged command before submitting another action.",
      );
    const cmd: Command = {
      id: crypto.randomUUID(),
      expected_revision: c.revision,
      action,
    };
    rememberPending(cmd, c.id);
    try {
      const result = await api.command(cmd);
      setRoom(result);
      rememberPending(null, c.id);
      return result;
    } catch (value) {
      const e = errorOf(value);
      if (!["timeout", "disconnected", "pending"].includes(e.code))
        rememberPending(null, c.id);
      throw e;
    }
  }
  const execute = (action: Action) =>
    run(async () => {
      await send(action);
    });
  async function syncSheets(choices: Record<string, string> = resolutions) {
    if (!campaign) return;
    setRetrySync(false);
    await run(async () => {
      try {
        const submitted = await api.prepareSync(
          selection,
          campaign.id,
          choices,
        );
        const accepted = await send({ type: "sync", sheets: submitted });
        await api.accept(
          accepted,
          submitted.map((s) => s.sheet.id),
          choices,
        );
        await refresh();
        setResolutions({});
        setToast("Personal and campaign sheets reconciled.");
      } catch (e) {
        if (errorOf(e).code === "conflict") setRetrySync(true);
        throw e;
      }
    });
  }
  const makeCopy = () =>
    run(async () => {
      if (!trainer) return;
      for (const sheet of cloneTrainer(library.sheets, trainer))
        await api.save({
          operation_id: crypto.randomUUID(),
          sheet,
          expected_revision: 0,
          xp_cost: 0,
          description: "Duplicated trainer and squad",
        });
      await refresh();
      setToast("Trainer and squad copied with new identities.");
    });
  if (!catalog)
    return (
      <div className="loading">
        <div className="brand-mark">P</div>
        <h2>Opening your field journal…</h2>
        {error && <p>{error.message}</p>}
      </div>
    );
  return (
    <div className="app-shell">
      <WorkspaceRail
        page={page}
        navigate={setPage}
        connected={connected}
        desktop={desktop}
      />
      <div className="workspace">
        <WorkspaceToolbar
          page={page}
          campaign={campaign}
          playerId={library.player_id}
          connected={connected}
          avatar={trainer?.name[0]?.toUpperCase() ?? "P"}
        />
        <main
          className={page === "Compendium" ? "compendium-main" : undefined}
          ref={mainRef}
        >
          {!desktop && (
            <div className="notice">
              Browser preview. Launch with <code>npm run desktop</code> to use
              SQLite storage and multiplayer rooms.
            </div>
          )}
          {error && (
            <div className="error" role="alert">
              <div>
                <strong>
                  {error.code === "conflict"
                    ? "Reconciliation needs your choice"
                    : "Could not complete this action"}
                </strong>
                <p>{error.message}</p>
                {error.conflicts.map((path) => (
                  <label className="conflict" key={path}>
                    {path}
                    <select
                      value={resolutions[path] ?? ""}
                      onChange={(e) =>
                        setResolutions((r) => ({
                          ...r,
                          [path]: e.target.value,
                        }))
                      }
                    >
                      <option value="">Choose value</option>
                      <option value="local">Personal value</option>
                      <option value="remote">Campaign value</option>
                    </select>
                  </label>
                ))}
                {retrySync && error.conflicts.length > 0 && (
                  <button
                    disabled={
                      busy || error.conflicts.some((p) => !resolutions[p])
                    }
                    onClick={() => syncSheets()}
                  >
                    Reconcile with these choices
                  </button>
                )}
              </div>
              <button
                className="icon"
                aria-label="Dismiss error"
                onClick={() => setError(null)}
              >
                <X size={18} />
              </button>
            </div>
          )}
          {campaign && !connected && (
            <div className="notice">
              <WifiOff size={16} />
              {connectionMessage || "Host unavailable; battle is paused."}
            </div>
          )}
          {pending && campaign && (
            <div className="notice">
              <div>
                <strong>Unacknowledged action retained.</strong> Retry its
                original ID to discover whether the host committed it. No new
                actions can be submitted yet.
              </div>
              <button
                disabled={busy || !connected}
                onClick={() =>
                  run(async () => {
                    try {
                      const result = await api.command(pending);
                      setRoom(result);
                      rememberPending(null, campaign.id);
                      setToast("Original command acknowledged.");
                    } catch (e) {
                      if (
                        !["timeout", "disconnected", "pending"].includes(
                          errorOf(e).code,
                        )
                      )
                        rememberPending(null, campaign.id);
                      throw e;
                    }
                  })
                }
              >
                Retry original action
              </button>
            </div>
          )}
          {page === "Library" && (
            <>
              {!!drafts.length && (
                <div className="panel">
                  <h3>Saved creation & progression drafts</h3>
                  {drafts.map((d) => (
                    <div className="list-row" key={d.operation_id}>
                      <span>
                        {d.sheet.name || "Unnamed"} · {d.kind}
                      </span>
                      <div className="inline">
                        <button
                          onClick={() => {
                            setEditingDraft(d);
                            setEditor(d.sheet);
                          }}
                        >
                          Resume
                        </button>
                        <button
                          disabled={busy}
                          onClick={() =>
                            run(async () => {
                              await call("delete_workflow_draft", {
                                operationId: d.operation_id,
                              });
                              await refresh();
                            })
                          }
                        >
                          Delete
                        </button>
                      </div>
                    </div>
                  ))}
                  <p>
                    Drafts do not replace accepted sheets. Submit narrative
                    requests from the room.
                  </p>
                </div>
              )}
              <div className="page-head">
                <div>
                  <span className="eyebrow">YOUR ADVENTURE STARTS HERE</span>
                  <h1>A partner for every journey.</h1>
                  <p>Your trainers and Pokémon, always close at hand.</p>
                </div>
                <button
                  className="primary"
                  onClick={() =>
                    setEditor(newSheet(library.player_id, "trainer"))
                  }
                >
                  <Plus size={17} /> New trainer
                </button>
              </div>
              <div className="hero">
                <div>
                  <span className="eyebrow">BUILT FOR YOUR NEXT SESSION</span>
                  <h2>
                    Train on your own.
                    <br />
                    Adventure together.
                  </h2>
                  <p>
                    Build your squad offline, then bring it into a shared room
                    <br className="wide" /> when your group is ready to play.
                  </p>
                  <button onClick={() => setPage("Room")}>
                    Open a room <ArrowUpRight size={17} />
                  </button>
                </div>
                <div className="hero-art">
                  <div className="orbit" />
                  <img
                    className="hero-pokemon one"
                    src="/sprites/bulbasaur.png"
                    alt="Bulbasaur"
                  />
                  <img
                    className="hero-pokemon two"
                    src="/sprites/charmander.png"
                    alt="Charmander"
                  />
                  <img
                    className="hero-pokemon three"
                    src="/sprites/squirtle.png"
                    alt="Squirtle"
                  />
                  <span className="hero-tag">
                    <Leaf size={15} /> A little progress, every day.
                  </span>
                </div>
              </div>
              <div className="section-head">
                <h2>
                  Your trainers <span>{trainers.length}</span>
                </h2>
                <label className="checkbox muted">
                  <input
                    type="checkbox"
                    checked={showArchived}
                    onChange={(e) => setShowArchived(e.target.checked)}
                  />{" "}
                  Show archived
                </label>
              </div>
              <div className="trainer-grid">
                {trainers.map((t) => (
                  <button
                    key={t.id}
                    onClick={() => setTrainerId(t.id)}
                    className={`trainer-card ${trainer?.id === t.id ? "active" : ""}`}
                  >
                    <div className="trainer-avatar">{t.name[0]}</div>
                    <div>
                      <strong>{t.name}</strong>
                      <small>
                        {t.rank} ·{" "}
                        {
                          library.sheets.filter(
                            (s) => s.trainer_id === t.id && !s.archived,
                          ).length
                        }{" "}
                        Pokémon{t.archived ? " · Archived" : ""}
                      </small>
                    </div>
                    {trainer?.id === t.id && <Check size={17} />}
                  </button>
                ))}
              </div>
              {!trainers.length && (
                <div className="empty">
                  <Users size={28} />
                  <h3>Make your first trainer</h3>
                  <p>
                    Your trainer is the home for your squad, inventory, and
                    story.
                  </p>
                  <button
                    onClick={() =>
                      setEditor(newSheet(library.player_id, "trainer"))
                    }
                  >
                    Create trainer
                  </button>
                </div>
              )}
              {trainer && (
                <>
                  <div className="section-head">
                    <div>
                      <h2>
                        {trainer.name}’s squad <span>{squad.length}</span>
                      </h2>
                      <p className="muted">
                        Personal sheets · saved on this device
                      </p>
                    </div>
                    <div className="inline">
                      <button
                        className="icon"
                        title="Copy trainer and squad"
                        aria-label="Copy trainer and squad"
                        disabled={busy}
                        onClick={makeCopy}
                      >
                        <Copy size={17} />
                      </button>
                      <button
                        onClick={() => setEditor(structuredClone(trainer))}
                      >
                        Edit trainer
                      </button>
                      <button
                        className="primary"
                        disabled={trainer.archived}
                        onClick={() =>
                          setEditor(
                            newSheet(library.player_id, "pokemon", trainer.id),
                          )
                        }
                      >
                        <Plus size={16} /> Add Pokémon
                      </button>
                    </div>
                  </div>
                  <div className="toolbar">
                    <div className="search">
                      <Search size={16} />
                      <input
                        aria-label="Search squad"
                        placeholder="Find a partner…"
                        value={search}
                        onChange={(e) => setSearch(e.target.value)}
                      />
                    </div>
                  </div>
                  <TrainingPointPool
                    key={trainer.id}
                    trainer={trainer}
                    pokemon={library.sheets.filter(
                      (s) =>
                        s.kind === "pokemon" &&
                        s.trainer_id === trainer.id &&
                        !s.archived,
                    )}
                    catalogHash={catalog.hash}
                    onChanged={refresh}
                    disabled={busy || !desktop}
                  />
                  <div className="pokemon-grid">
                    {squad.map((p) => {
                      const species = catalog.species.find(
                        (s) => s.AppId === p.species_id,
                      );
                      return (
                        <button
                          className="pokemon-card"
                          key={p.id}
                          onClick={() => setEditor(structuredClone(p))}
                        >
                          <div className="inline between">
                            <span className="pill">{p.rank}</span>
                            <span className="muted">
                              {p.archived
                                ? "Archived"
                                : `#${species?.DexID ?? "—"}`}
                            </span>
                          </div>
                          <div className="pokemon-art">
                            <img
                              src={`/sprites/${species?.Image}`}
                              alt=""
                              onError={(e) =>
                                (e.currentTarget.style.visibility = "hidden")
                              }
                            />
                          </div>
                          <h3>{p.name}</h3>
                          <p>{species?.Name}</p>
                          <div className="type-row">
                            <span className="type">{species?.Type1}</span>
                            {species?.Type2 && (
                              <span className="type">{species.Type2}</span>
                            )}
                          </div>
                          <div className="card-footer">
                            <span>
                              HP{" "}
                              <strong>
                                {p.hp}/{p.max_hp}
                              </strong>
                            </span>
                            <span>
                              TP{" "}
                              <strong>
                                {p.workflow?.training_points ?? p.xp}
                              </strong>
                            </span>
                            <ArrowUpRight size={16} />
                          </div>
                        </button>
                      );
                    })}
                    <button
                      className="add-card"
                      disabled={trainer.archived}
                      onClick={() =>
                        setEditor(
                          newSheet(library.player_id, "pokemon", trainer.id),
                        )
                      }
                    >
                      <div>
                        <Plus size={25} />
                      </div>
                      <strong>A new companion</strong>
                      <span>Grow your squad</span>
                    </button>
                  </div>
                </>
              )}
            </>
          )}
          {page === "Compendium" && <CatalogView catalog={catalog} />}
          {page === "Room" && (
            <>
              <div className="page-head">
                <div>
                  <span className="eyebrow">ONE HOST · ONE SHARED STORY</span>
                  <h1>{campaign ? campaign.name : "Gather your party."}</h1>
                  <p>
                    {campaign
                      ? `Campaign revision ${campaign.revision} · ${(campaign.host_id || campaign.gm_id) === library.player_id ? "Host · " : ""}${campaign.gm_id === library.player_id ? "You are the GM" : "You are a player"}`
                      : "Host on your computer or join your group over LAN / private VPN."}
                  </p>
                </div>
                {campaign && (
                  <button
                    disabled={busy}
                    onClick={() =>
                      run(async () => {
                        await api.leave();
                        setCampaign(null);
                        campaignRef.current = null;
                        setRoomInfo(null);
                        setConnected(false);
                        setPending(null);
                        await refresh();
                      })
                    }
                  >
                    Leave room
                  </button>
                )}
              </div>
              {!campaign ? (
                <div className="two-columns">
                  <div className="panel">
                    <Radio size={26} />
                    <h2>Host a room</h2>
                    <p className="muted">
                      Your computer keeps the campaign and validates all
                      actions. You can appoint another player as the GM.
                    </p>
                    <label>
                      New campaign name
                      <input
                        value={campaignName}
                        onChange={(e) => setCampaignName(e.target.value)}
                        placeholder="The Kanto expedition"
                        maxLength={100}
                      />
                    </label>
                    <button
                      disabled={busy || !campaignName.trim()}
                      onClick={() =>
                        run(async () => {
                          const c = await api.createCampaign(campaignName);
                          await refresh();
                          setCampaignId(c.id);
                          setCampaignName("");
                        })
                      }
                    >
                      <Plus size={16} /> Create campaign
                    </button>
                    <label>
                      Campaign
                      <select
                        value={campaignId}
                        onChange={(e) => setCampaignId(e.target.value)}
                      >
                        <option value="">Select campaign</option>
                        {campaigns
                          .filter(
                            (c) => (c.host_id || c.gm_id) === library.player_id,
                          )
                          .map((c) => (
                            <option key={c.id} value={c.id}>
                              {c.name} · revision {c.revision}
                            </option>
                          ))}
                      </select>
                    </label>
                    <label>
                      Offline preparation
                      <select
                        value=""
                        onChange={(e) => {
                          const cached = campaigns.find(
                            (c) => c.id === e.target.value,
                          );
                          if (cached) {
                            setRoom(cached);
                            setConnected(false);
                            setRoomInfo(null);
                            setSelection([]);
                            loadPending(cached.id);
                          }
                        }}
                      >
                        <option value="">
                          Open a saved campaign without connecting…
                        </option>
                        {campaigns.map((c) => (
                          <option key={c.id} value={c.id}>
                            {c.name} · revision {c.revision}
                          </option>
                        ))}
                      </select>
                    </label>
                    <label>
                      Port
                      <input
                        type="number"
                        min={1024}
                        max={65535}
                        value={port}
                        onChange={(e) => setPort(Number(e.target.value))}
                      />
                    </label>
                    <button
                      className="primary"
                      disabled={busy || !campaignId}
                      onClick={() =>
                        run(async () => {
                          const info = await api.host(campaignId, port);
                          setRoomInfo(info);
                          const c = campaigns.find((c) => c.id === campaignId)!;
                          setRoom(c);
                          setConnected(true);
                          loadPending(c.id);
                          setSelection([]);
                        })
                      }
                    >
                      Start hosting <ArrowUpRight size={16} />
                    </button>
                  </div>
                  <div className="panel">
                    <Users size={26} />
                    <h2>Join your group</h2>
                    <p className="muted">
                      Ask the host for their LAN/VPN address and access token.
                    </p>
                    <label>
                      Host address
                      <input
                        value={address}
                        onChange={(e) => setAddress(e.target.value)}
                      />
                    </label>
                    <label>
                      Room access token
                      <input
                        value={token}
                        onChange={(e) => setToken(e.target.value)}
                        autoComplete="off"
                      />
                    </label>
                    <button
                      className="primary"
                      disabled={busy || !address || !token}
                      onClick={() =>
                        run(async () => {
                          const c = await api.join(address, token);
                          setRoom(c);
                          setConnected(true);
                          setRoomInfo(null);
                          loadPending(c.id);
                          setSelection([]);
                        })
                      }
                    >
                      Join room <ArrowUpRight size={16} />
                    </button>
                    <p className="notice">
                      No cloud account required. The host must allow this app
                      through Windows Firewall on the private network. Public
                      internet hosting is outside v1.
                    </p>
                  </div>
                </div>
              ) : (
                <>
                  {roomInfo && (
                    <div className="panel room-share">
                      <div>
                        <span className="eyebrow">SHARE WITH YOUR GROUP</span>
                        <h3>Room access</h3>
                        <p>
                          Replace <code>&lt;host-ip&gt;</code> with your LAN or
                          VPN IPv4 address.
                        </p>
                        <code>{roomInfo.address}</code>
                      </div>
                      <div>
                        <small>Access token</small>
                        <code>{roomInfo.token}</code>
                        <button
                          onClick={() =>
                            run(async () => {
                              await navigator.clipboard.writeText(
                                roomInfo.token,
                              );
                              setToast("Room token copied.");
                            })
                          }
                        >
                          <Copy size={15} /> Copy token
                        </button>
                      </div>
                    </div>
                  )}
                  <div className="panel">
                    <div className="section-head">
                      <div>
                        <h2>Bring your sheets</h2>
                        <p className="muted">
                          Valid player changes are trusted. Active combatants
                          stay frozen until battle ends.
                        </p>
                      </div>
                      <button
                        onClick={() =>
                          setSelection(
                            library.sheets
                              .filter((s) => !s.archived)
                              .map((s) => s.id),
                          )
                        }
                      >
                        Select all active
                      </button>
                    </div>
                    {library.sheets
                      .filter((s) => !s.archived)
                      .map((s) => (
                        <div className="list-row" key={s.id}>
                          <label className="checkbox">
                            <input
                              type="checkbox"
                              checked={selection.includes(s.id)}
                              onChange={() =>
                                setSelection((ids) =>
                                  ids.includes(s.id)
                                    ? ids.filter((x) => x !== s.id)
                                    : [...ids, s.id],
                                )
                              }
                            />
                            {s.name} <span className="muted">{s.kind}</span>
                          </label>
                          <span className="pill">
                            {campaign.sheets[s.id]
                              ? "In campaign"
                              : "Personal only"}
                          </span>
                        </div>
                      ))}
                    <div className="inline">
                      <button
                        className="primary"
                        disabled={
                          busy || !connected || !!pending || !selection.length
                        }
                        onClick={() => syncSheets()}
                      >
                        <RefreshCw size={16} /> Reconcile selected sheets
                      </button>
                      <button
                        disabled={busy || !selection.length}
                        onClick={() =>
                          run(async () => {
                            await api.accept(campaign, selection, resolutions);
                            await refresh();
                            setToast(
                              "Campaign outcomes pulled into the personal library.",
                            );
                          })
                        }
                      >
                        Pull campaign outcomes
                      </button>
                    </div>
                    <p className="muted">
                      Include the trainer with new Pokémon. Duplicate a trainer
                      and squad before using them in another campaign.
                    </p>
                  </div>
                  <div className="panel">
                    <h2>Campaign roster</h2>
                    <WorkflowRoom
                      campaign={campaign}
                      drafts={drafts}
                      playerId={library.player_id}
                      disabled={busy || !connected || !!pending}
                      execute={execute}
                      pull={async (ids) => {
                        await run(async () => {
                          await api.accept(campaign, ids, resolutions);
                          await refresh();
                        });
                      }}
                    />
                    {sheetValues(campaign.sheets).map((s) => (
                      <div className="list-row" key={s.id}>
                        <div>
                          <strong>{s.name}</strong>
                          <small>
                            {s.owner_id === library.player_id
                              ? "Your sheet"
                              : "Other player"}{" "}
                            · {s.kind} · {s.rank}
                          </small>
                        </div>
                        <span>
                          HP {s.hp}/{s.max_hp} · TP{" "}
                          {s.workflow?.training_points ??
                            (s.kind === "trainer" ? 0 : s.xp)}
                        </span>
                      </div>
                    ))}
                    <button onClick={() => setPage("Battle")}>
                      Open battle control <Swords size={16} />
                    </button>
                  </div>
                  {roomInfo &&
                    (campaign.host_id || campaign.gm_id) ===
                      library.player_id && (
                      <GmAssignment
                        campaign={campaign}
                        disabled={busy || !connected || !!pending}
                        execute={execute}
                      />
                    )}
                  {campaign.gm_id === library.player_id && (
                    <div className="panel">
                      <h3>Campaign rewards</h3>
                      <div className="form-grid">
                        <label>
                          Trainer
                          <select
                            value={rewardSheet}
                            onChange={(e) => setRewardSheet(e.target.value)}
                          >
                            <option value="">Select recipient</option>
                            {sheetValues(campaign.sheets)
                              .filter(
                                (s) => s.kind === "trainer" && !s.archived,
                              )
                              .map((s) => (
                                <option key={s.id} value={s.id}>
                                  {s.name}
                                </option>
                              ))}
                          </select>
                        </label>
                        <label>
                          Training Points
                          <input
                            type="number"
                            min={1}
                            value={rewardXP}
                            onChange={(e) =>
                              setRewardXP(Number(e.target.value))
                            }
                          />
                        </label>
                        <label>
                          Reason
                          <input
                            value={rewardReason}
                            onChange={(e) => setRewardReason(e.target.value)}
                          />
                        </label>
                      </div>
                      <button
                        disabled={
                          busy ||
                          !connected ||
                          !!pending ||
                          !rewardSheet ||
                          rewardXP <= 0 ||
                          !rewardReason.trim() ||
                          !!campaign.battle?.active
                        }
                        onClick={() =>
                          execute({
                            type: "reward",
                            sheet_id: rewardSheet,
                            xp: rewardXP,
                            reason: rewardReason,
                          })
                        }
                      >
                        Award Training Points
                      </button>
                    </div>
                  )}
                </>
              )}
            </>
          )}
          {page === "Training encounters" && (
            <TrainingEncounters
              campaign={campaign}
              catalog={catalog}
              playerId={library.player_id}
              execute={execute}
              connected={connected}
              busy={busy || !!pending}
              pull={async (ids) => {
                if (campaign) {
                  await api.accept(campaign, ids, {});
                  await refresh();
                  setToast(
                    "Captured Pokémon imported into your offline library.",
                  );
                }
              }}
            />
          )}
          {page === "Battle" && (
            <BattleView
              error={error}
              campaign={campaign}
              playerId={library.player_id}
              catalog={catalog}
              execute={execute}
              disabled={busy || !connected || !!pending}
            />
          )}
          {page === "History" && (
            <>
              <div className="page-head">
                <div>
                  <span className="eyebrow">EVERY STEP COUNTS</span>
                  <h1>Your progression journal</h1>
                  <p>
                    Saved edits, training, and campaign reconciliation on this
                    device.
                  </p>
                </div>
              </div>
              <div className="panel">
                {[...library.history].reverse().map((h) => (
                  <div className="list-row" key={h.id}>
                    <div>
                      <strong>{h.after.name}</strong>
                      <p>{h.description}</p>
                      <small>
                        Revision {h.revision} · Points spent {h.xp_cost}
                      </small>
                    </div>
                    <span className="pill">{h.after.rank}</span>
                  </div>
                ))}
                {!library.history.length && (
                  <div className="empty">
                    <History size={30} />
                    <h3>Your story is still unwritten.</h3>
                    <p>Create a trainer to begin your journal.</p>
                  </div>
                )}
              </div>
            </>
          )}
          {page === "Settings" && (
            <>
              <div className="page-head">
                <div>
                  <span className="eyebrow">YOUR DATA, ON YOUR COMPUTER</span>
                  <h1>Workspace settings</h1>
                  <p>
                    Backups, portable sheets, and the rules version behind your
                    campaign.
                  </p>
                </div>
              </div>
              <div className="two-columns">
                <div className="panel">
                  <h2>Personal library</h2>
                  <p>
                    Exports are portable JSON. Import creates new identities so
                    sheets never overwrite existing characters.
                  </p>
                  <div className="inline">
                    <button
                      disabled={busy}
                      onClick={() =>
                        run(async () => {
                          download(
                            "pokeverse-library.json",
                            await call<string>("export_library"),
                          );
                        })
                      }
                    >
                      <Download size={16} /> Export
                    </button>
                    <button
                      disabled={busy}
                      onClick={() => importRef.current?.click()}
                    >
                      <Upload size={16} /> Import
                    </button>
                    <input
                      hidden
                      type="file"
                      accept=".json,application/json"
                      ref={importRef}
                      onChange={(e) => {
                        const file = e.target.files?.[0];
                        if (file)
                          run(async () => {
                            await call("import_library", {
                              text: await file.text(),
                            });
                            await refresh();
                            setToast(
                              "Library imported as new character copies.",
                            );
                          });
                        e.target.value = "";
                      }}
                    />
                  </div>
                  <h3>Local backups</h3>
                  <p>
                    Consistent SQLite backups include campaigns, memberships,
                    and reconnect credentials.
                  </p>
                  <button
                    disabled={busy}
                    onClick={() =>
                      run(async () => {
                        const path = await call<string>("backup");
                        setBackups(await call<string[]>("list_backups"));
                        setToast(`Backup saved: ${path}`);
                      })
                    }
                  >
                    <Download size={16} /> Create backup
                  </button>
                  <label>
                    Backup to restore
                    <select
                      value={restoreId}
                      onChange={(e) => {
                        setRestoreId(e.target.value);
                        setConfirmRestore(false);
                      }}
                    >
                      <option value="">Select backup</option>
                      {backups.map((id) => (
                        <option key={id}>{id}</option>
                      ))}
                    </select>
                  </label>
                  <label className="checkbox">
                    <input
                      type="checkbox"
                      checked={confirmRestore}
                      onChange={(e) => setConfirmRestore(e.target.checked)}
                    />{" "}
                    Replace current data with this backup
                  </label>
                  <button
                    disabled={
                      busy || !!campaign || !restoreId || !confirmRestore
                    }
                    onClick={() =>
                      run(async () => {
                        await call("restore_backup", { backupId: restoreId });
                        await refresh();
                        setBackups(await call<string[]>("list_backups"));
                        setConfirmRestore(false);
                        setToast(
                          "Backup restored. A safety backup was created first.",
                        );
                      })
                    }
                  >
                    Restore selected backup
                  </button>
                </div>
                <div className="panel">
                  <h2>Rules & compatibility</h2>
                  <div className="list-row">
                    <span>Edition</span>
                    <strong>Pokerole {catalog.edition}</strong>
                  </div>
                  <div className="list-row">
                    <span>Engine</span>
                    <strong>0.8.0 · Special moves and battle automation</strong>
                  </div>
                  <div className="list-row">
                    <span>Transport</span>
                    <strong>LAN / private VPN</strong>
                  </div>
                  <div className="list-row">
                    <span>Storage</span>
                    <strong>Local SQLite</strong>
                  </div>
                  <label>
                    Catalog fingerprint
                    <code className="break">{catalog.hash}</code>
                  </label>
                  <div className="notice">
                    Shared Rust rules calculate supported rolls, derived stats,
                    turn order, reactions, Will and reviewed status/effect
                    definitions. Unreviewed effects and conflicting manual
                    entries require recorded GM adjudication. Compendium
                    warnings distinguish verified fields from resolved
                    references.
                  </div>
                  <details>
                    <summary>
                      {catalog.warnings.length} source reference warnings
                    </summary>
                    {catalog.warnings.map((w, i) => (
                      <p className="muted" key={i}>
                        {w}
                      </p>
                    ))}
                  </details>
                </div>
              </div>
            </>
          )}
        </main>
        <footer className="app-footer">
          <span>
            <span className="dot online" /> Your library stays with you.
          </span>
          <span>POKEROLE 3.0 · v0.15.0</span>
        </footer>
      </div>
      {editor && (
        <SheetEditor
          initial={editor}
          catalog={catalog}
          trainers={library.sheets.filter(
            (s) => s.kind === "trainer" && !s.archived,
          )}
          draftRequest={editingDraft}
          close={() => {
            setEditor(null);
            setEditingDraft(null);
          }}
          saved={async () => {
            await refresh();
            setToast("Workflow saved to your personal library.");
          }}
        />
      )}
      {toast && (
        <div className="toast" role="status">
          <Check size={18} />
          {toast}
        </div>
      )}
    </div>
  );
}
