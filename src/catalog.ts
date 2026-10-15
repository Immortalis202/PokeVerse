export type Species = {
  AppId: string;
  DexID: string;
  Name: string;
  Type1: string;
  Type2: string;
  BaseHP: number;
  Ability1: string;
  Ability2: string;
  HiddenAbility: string;
  RecommendedRank: string;
  Image: string;
  DexDescription: string;
  Moves: { Learned: string; Name: string }[];
  Creation?: {
    stage: number | null;
    audit: string;
    pdf_page?: number;
    unresolved_moves?: string[];
    unresolved_fields?: string[];
    baselines: Record<string, number>;
    limits: Record<string, number>;
  };
  Evolutions: {
    To?: string;
    From?: string;
    Kind: string;
    Item?: string;
    Value?: number;
    Stat?: string;
  }[];
  [key: string]: unknown;
};
export type Move = {
  [key: string]: unknown;
  AppId: string;
  Name: string;
  Type: string;
  Power: number;
  Category: string;
  Target: string;
  Accuracy1: string;
  Accuracy2: string;
  Damage1: string;
  Damage2: string;
  Effect: string;
  Description: string;
};
export type CatalogEntry = {
  Name: string;
  Effect?: string;
  Description: string;
  [key: string]: unknown;
};
export type Catalog = {
  edition: string;
  hash: string;
  species: SpeciesSummary[];
  moves: MoveSummary[];
  abilities: EntrySummary[];
  natures: EntrySummary[];
  items: EntrySummary[];
  warnings: string[];
};
export const attributes = [
  "Strength",
  "Dexterity",
  "Vitality",
  "Special",
  "Insight",
];
export const ranks = [
  "Starter",
  "Rookie",
  "Standard",
  "Advanced",
  "Expert",
  "Ace",
  "Master",
  "Champion",
];

export type CatalogKind =
  "species" | "moves" | "abilities" | "items" | "natures";
export type SpeciesSummary = Pick<
  Species,
  | "AppId"
  | "DexID"
  | "Name"
  | "Type1"
  | "Type2"
  | "Image"
  | "Ability1"
  | "Ability2"
> & { RecommendedRank?: string; Legendary?: boolean; GoodStarter?: boolean };
export type MoveSummary = Pick<Move, "AppId" | "Name" | "Type" | "Category">;
export type EntrySummary = { AppId: string; Name: string };
export type CatalogSummary = SpeciesSummary | MoveSummary | EntrySummary;
export type CatalogRecord = Species | Move | CatalogEntry;
export type CatalogPage = { entries: CatalogSummary[]; total: number };
