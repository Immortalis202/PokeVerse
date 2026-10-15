import type { PointTransferRequest } from "./bindings/PointTransferRequest";
import type { PointTransferResult } from "./bindings/PointTransferResult";
import type { CompendiumDetail } from "./bindings/CompendiumDetail";
import { invoke, isTauri } from "@tauri-apps/api/core";
import type { Library } from "./bindings/Library";
import type { Sheet } from "./bindings/Sheet";
import type { SaveRequest } from "./bindings/SaveRequest";
import type { Campaign } from "./bindings/Campaign";
import type { Command } from "./bindings/Command";
import type { RoomInfo } from "./bindings/RoomInfo";
import type { SyncSheet } from "./bindings/SyncSheet";
import type { AppError } from "./bindings/AppError";
import type {
  Catalog,
  CatalogKind,
  CatalogRecord,
  CatalogPage,
} from "./catalog";
import type { WorkflowRequest } from "./bindings/WorkflowRequest";
import type { WorkflowPreview } from "./bindings/WorkflowPreview";
import type { TrainingEncounter } from "./bindings/TrainingEncounter";
import type { CapturePreview } from "./bindings/CapturePreview";
export const desktop = isTauri();
export async function call<T>(
  name: string,
  args: Record<string, unknown> = {},
): Promise<T> {
  if (!desktop)
    throw {
      code: "desktop",
      message: "Open the desktop app to save sheets or connect to rooms.",
      conflicts: [],
    } satisfies AppError;
  return invoke<T>(name, args);
}
export const api = {
  previewBattleAction: (
    campaign: Campaign,
    action: import("./bindings/Action").Action,
  ) =>
    call<import("./bindings/ActionPreview").ActionPreview>(
      "preview_battle_action",
      { campaign, action },
    ),
  previewPointTransfer: (request: PointTransferRequest) =>
    call<PointTransferResult>("preview_point_transfer", { request }),
  commitPointTransfer: (request: PointTransferRequest) =>
    call<PointTransferResult>("commit_point_transfer", { request }),
  compendiumDetail: (kind: CatalogKind, id: string) =>
    catalogCall<CompendiumDetail | null>("compendium_detail", { kind, id }),
  encounterHabitats: () =>
    call<
      {
        habitat_id: string;
        habitat: string;
        description: string;
        pokemon_count: number;
      }[]
    >("encounter_habitats"),
  encounterPool: (habitat: string, query = "", offset = 0) =>
    call<{
      entries: (Catalog["species"][number] & {
        habitat_provenance: { conditions: string };
      })[];
      total: number;
    }>("encounter_pool", { habitat, query, offset }),
  generateEncounter: (
    campaignId: string,
    trainerId: string,
    habitat: string,
    rank: string,
    size: number,
    seed: string,
  ) =>
    call<TrainingEncounter>("generate_encounter", {
      campaignId,
      trainerId,
      habitat,
      rank,
      size,
      seed,
    }),
  encounterDrafts: () => call<TrainingEncounter[]>("encounter_drafts"),
  saveEncounterDraft: (campaign: Campaign, encounter: TrainingEncounter) =>
    call<void>("save_encounter_draft", { campaign, encounter }),
  deleteEncounterDraft: (encounterId: string) =>
    call<void>("delete_encounter_draft", { encounterId }),
  previewCapture: (
    campaign: Campaign,
    encounter: TrainingEncounter,
    individualId: string,
    trainerId: string,
    edited: WorkflowRequest | null = null,
  ) =>
    call<CapturePreview>("preview_capture", {
      campaign,
      encounter,
      individualId,
      trainerId,
      edited,
    }),
  previewWorkflow: (request: WorkflowRequest) =>
    call<WorkflowPreview>("preview_workflow", { request }),
  previewCampaignWorkflow: (campaign: Campaign, request: WorkflowRequest) =>
    call<WorkflowPreview>("preview_campaign_workflow", { campaign, request }),
  commitWorkflow: (request: WorkflowRequest) =>
    call<Sheet>("commit_workflow", { request }),
  saveDraft: (request: WorkflowRequest) =>
    call<void>("save_workflow_draft", { request }),
  drafts: () => call<WorkflowRequest[]>("list_workflow_drafts"),
  library: () => call<Library>("get_library"),
  catalog: () => catalogCall<Catalog>("get_catalog"),
  catalogEntry: <T extends CatalogRecord>(kind: CatalogKind, id: string) =>
    catalogCall<T | null>("catalog_entry", { kind, id }),
  searchCatalog: (
    kind: CatalogKind,
    query: string,
    typeFilter: string,
    offset = 0,
  ) =>
    catalogCall<CatalogPage>("search_catalog", {
      kind,
      query,
      typeFilter,
      offset,
    }),
  save: (request: SaveRequest) => call<Sheet>("save_sheet", { request }),
  campaigns: () => call<Campaign[]>("list_campaigns"),
  createCampaign: (name: string) => call<Campaign>("create_campaign", { name }),
  host: (campaignId: string, port: number) =>
    call<RoomInfo>("host_room", { campaignId, port }),
  join: (address: string, token: string) =>
    call<Campaign>("join_room", { request: { address, token } }),
  leave: () => call<void>("leave_room"),
  command: (command: Command) => call<Campaign>("room_command", { command }),
  prepareSync: (
    sheetIds: string[],
    campaignId: string,
    resolutions: Record<string, string>,
  ) => call<SyncSheet[]>("prepare_sync", { sheetIds, campaignId, resolutions }),
  accept: (
    campaign: Campaign,
    sheetIds: string[],
    resolutions: Record<string, string>,
  ) => call<void>("accept_campaign", { campaign, sheetIds, resolutions }),
};
export function errorOf(value: unknown): AppError {
  if (typeof value === "object" && value !== null && "message" in value)
    return { code: "error", conflicts: [], ...value } as AppError;
  return { code: "error", message: String(value), conflicts: [] };
}
export function download(name: string, text: string) {
  const url = URL.createObjectURL(
    new Blob([text], { type: "application/json" }),
  );
  const a = document.createElement("a");
  a.href = url;
  a.download = name;
  a.click();
  URL.revokeObjectURL(url);
}

// Development browser preview uses the same SQLite file through Vite's local API.
async function catalogCall<T>(
  name: string,
  args: Record<string, unknown> = {},
): Promise<T> {
  if (desktop) return call<T>(name, args);
  const response = await fetch(
    `/__catalog/${name}?${new URLSearchParams(Object.entries(args).map(([k, v]) => [k, String(v)]))}`,
  );
  if (!response.ok)
    throw new Error(
      "Catalog unavailable. Open the desktop app or development preview.",
    );
  return response.json();
}
