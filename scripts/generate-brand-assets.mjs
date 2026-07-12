#!/usr/bin/env node
/**
 * Build brand PNGs from static/brand/logo-symbol-source.png (symbol-only)
 * and static/brand/logo-full.png (wordmark for About page).
 * Run before `npm run icons` when artwork changes.
 */
import { execFileSync } from "node:child_process";
import { existsSync, copyFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const brand = join(root, "static/brand");
const symbol = join(brand, "logo-symbol-source.png");
const fullLogo = join(brand, "logo-full.png");

// Symbol is ~1.45:1 (wide). Scale to max width inside each square canvas.
const NAV_CANVAS = 256;
const NAV_WIDTH = 248;
const APP_CANVAS = 1024;
const APP_WIDTH = 1024;
const SRC_CANVAS = 512;
const SRC_WIDTH = 496;

if (!existsSync(symbol)) {
  console.error(
    "Missing static/brand/logo-symbol-source.png — add the symbol-only artwork there.",
  );
  process.exit(1);
}

function letterboxIcon(input, width, canvas, output) {
  execFileSync(
    "convert",
    [
      input,
      "-filter",
      "Lanczos",
      "-resize",
      `${width}x`,
      "-background",
      "black",
      "-gravity",
      "center",
      "-extent",
      `${canvas}x${canvas}`,
      output,
    ],
    { stdio: "inherit" },
  );
}

// Exact symbol crop (no re-trimming — source is already correct).
copyFileSync(symbol, join(brand, "trimmed.png"));

letterboxIcon(symbol, SRC_WIDTH, SRC_CANVAS, join(brand, "logo-icon-src.png"));
letterboxIcon(
  symbol,
  NAV_WIDTH,
  NAV_CANVAS,
  join(brand, "logo-icon-centered.png"),
);
copyFileSync(
  join(brand, "logo-icon-centered.png"),
  join(brand, "centered.png"),
);
copyFileSync(
  join(brand, "logo-icon-centered.png"),
  join(brand, "logo-icon.png"),
);

// Desktop / Android / .exe — fill the full canvas width for maximum presence.
letterboxIcon(
  symbol,
  APP_WIDTH,
  APP_CANVAS,
  join(brand, "app-icon-1024.png"),
);

execFileSync(
  "convert",
  [
    join(brand, "app-icon-1024.png"),
    "-resize",
    "256x256",
    join(root, "static/favicon.png"),
  ],
  { stdio: "inherit" },
);

// About-page wordmark (separate from the symbol).
if (existsSync(fullLogo)) {
  execFileSync(
    "convert",
    [
      fullLogo,
      "-fuzz",
      "10%",
      "-trim",
      "+repage",
      "-resize",
      "480x",
      join(brand, "logo-full-trimmed.png"),
    ],
    { stdio: "inherit" },
  );
}

console.log("Generated brand assets from logo-symbol-source.png");
