#!/usr/bin/env node
/**
 * Tauri icon generation resets Android launcher background to white.
 * Keep it black to match the Black Magic Software logo.
 */
import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const valuesXml = join(
  root,
  "src-tauri/gen/android/app/src/main/res/values/ic_launcher_background.xml",
);
const drawableXml = join(
  root,
  "src-tauri/gen/android/app/src/main/res/drawable/ic_launcher_background.xml",
);

const solidBlackDrawable = `<?xml version="1.0" encoding="utf-8"?>
<vector xmlns:android="http://schemas.android.com/apk/res/android"
    android:width="108dp"
    android:height="108dp"
    android:viewportWidth="108"
    android:viewportHeight="108">
    <path
        android:fillColor="#000000"
        android:pathData="M0,0h108v108h-108z" />
</vector>
`;

try {
  if (!existsSync(valuesXml)) {
    console.warn(
      "Android icon background patch skipped: gen/android not found (run after tauri android init)",
    );
    process.exit(0);
  }

  let values = readFileSync(valuesXml, "utf8");
  values = values.replace(
    /<color name="ic_launcher_background">[^<]+<\/color>/,
    '<color name="ic_launcher_background">#000000</color>',
  );
  writeFileSync(valuesXml, values);
  writeFileSync(drawableXml, solidBlackDrawable);
  console.log("Patched Android launcher background to #000000");
} catch (err) {
  console.warn("Android icon background patch skipped:", err.message);
}
