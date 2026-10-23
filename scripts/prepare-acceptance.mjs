import {
  readFileSync,
  writeFileSync,
  mkdirSync,
  copyFileSync,
  statSync,
} from "node:fs";
import { createHash, randomUUID } from "node:crypto";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";

const workspace = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
);
const readJson = (file) =>
  JSON.parse(readFileSync(path.join(workspace, file), "utf8"));
const hash = (file) =>
  createHash("sha256").update(readFileSync(file)).digest("hex");
const version = readJson("package.json").version;
if (readJson("src-tauri/tauri.conf.json").version !== version)
  throw new Error("Package and Tauri versions disagree");
if (process.platform !== "win32")
  throw new Error("Prepare this Windows acceptance lab on Windows");
const release = path.join(workspace, "src-tauri/target/release");
const files = {
  executable: path.join(release, "pokeverse.exe"),
  catalog: path.join(release, "resources/catalog.db"),
  installer: path.join(
    release,
    `bundle/nsis/PokeVerse_${version}_x64-setup.exe`,
  ),
};
const artifacts = Object.fromEntries(
  Object.entries(files).map(([name, file]) => [
    name,
    { path: file, bytes: statSync(file).size, sha256: hash(file) },
  ]),
);
// Read Windows version/signature metadata with a fixed command. No policy changes,
// installer execution or app launch; paths are passed as environment values.
let metadataText;
for (const shell of ["pwsh.exe", "powershell.exe"]) {
  try {
    metadataText = execFileSync(
      shell,
      [
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        "$ErrorActionPreference='Stop'; $taskExe=Get-Item -LiteralPath $env:POKEVERSE_ACCEPTANCE_EXE; $taskStatus='Unavailable'; try { $taskStatus=(Get-AuthenticodeSignature -LiteralPath $env:POKEVERSE_ACCEPTANCE_INSTALLER).Status.ToString() } catch {}; @{productVersion=$taskExe.VersionInfo.ProductVersion; signature=$taskStatus} | ConvertTo-Json -Compress",
      ],
      {
        encoding: "utf8",
        windowsHide: true,
        env: {
          ...process.env,
          POKEVERSE_ACCEPTANCE_EXE: files.executable,
          POKEVERSE_ACCEPTANCE_INSTALLER: files.installer,
        },
      },
    );
    break;
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
  }
}
if (!metadataText)
  throw new Error("Windows PowerShell is required to read executable metadata");
const metadata = JSON.parse(metadataText);
if (metadata.productVersion !== version)
  throw new Error(
    `Built executable version ${metadata.productVersion} does not match ${version}; run npm run package`,
  );
const lab = path.join(
  workspace,
  "tmp/acceptance",
  `${version}-${randomUUID()}`,
);
mkdirSync(lab, { recursive: true });
const roles = ["host", "gm", "owner1", "owner2", "owner3"];
for (const role of roles) {
  const installation = path.join(lab, role);
  mkdirSync(path.join(installation, "portable-data"), { recursive: true });
  mkdirSync(path.join(installation, "resources"));
  const executable = path.join(installation, "pokeverse.exe");
  const catalog = path.join(installation, "resources/catalog.db");
  copyFileSync(files.executable, executable);
  copyFileSync(files.catalog, catalog);
  if (
    hash(executable) !== artifacts.executable.sha256 ||
    hash(catalog) !== artifacts.catalog.sha256
  )
    throw new Error(`Artifact copy failed: ${role}`);
}
const git = (...args) =>
  execFileSync("git", ["-C", workspace, ...args], {
    encoding: "utf8",
    windowsHide: true,
  }).trim();
writeFileSync(
  path.join(lab, "manifest.json"),
  JSON.stringify(
    {
      format: 1,
      prepared_utc: new Date().toISOString(),
      version,
      source_commit: git("rev-parse", "HEAD"),
      source_dirty: !!git("status", "--porcelain"),
      executable_product_version: metadata.productVersion,
      installer_signature: metadata.signature,
      artifacts,
      profiles: roles.map((role) => ({
        role,
        directory: role,
        status: "prepared-not-launched",
      })),
      acceptance: "pending-manual-execution",
    },
    null,
    2,
  ) + "\n",
  { flag: "wx" },
);
copyFileSync(
  path.join(workspace, "docs/acceptance-results-template.csv"),
  path.join(lab, "results.csv"),
);
console.log(`Prepared isolated profiles: ${lab}`);
console.log(
  `Installer signature: ${metadata.signature}. No application or installer was launched.`,
);
console.log(
  "Read docs/windows-lan-acceptance.md before starting the manual cases.",
);
