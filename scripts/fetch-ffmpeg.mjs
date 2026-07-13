import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";

const root = path.dirname(path.dirname(fileURLToPath(import.meta.url)));

const result =
  process.platform === "win32"
    ? spawnSync(
        "powershell",
        [
          "-ExecutionPolicy",
          "Bypass",
          "-File",
          path.join(root, "scripts", "fetch-ffmpeg-windows.ps1"),
        ],
        { cwd: root, stdio: "inherit" },
      )
    : spawnSync("bash", [path.join(root, "scripts", "fetch-ffmpeg-windows.sh")], {
        cwd: root,
        stdio: "inherit",
      });

process.exit(result.status ?? 1);
