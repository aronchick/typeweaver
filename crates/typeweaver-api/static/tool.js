const STARTER_FONTS = [
  {
    id: "roboto",
    label: "Roboto",
    family: "Roboto",
    familyQuery: "Roboto:wght@400;700",
    fallback: "system-ui, sans-serif",
    description: "Neutral default for product UI.",
    uploadFileName: "Roboto-Regular.ttf",
    licenseText: "Apache License Version 2.0",
  },
  {
    id: "roboto_condensed",
    label: "Roboto Condensed",
    family: "Roboto Condensed",
    familyQuery: "Roboto+Condensed:wght@400;700",
    fallback: "\"Arial Narrow\", sans-serif",
    description: "Tighter read for dense navigation.",
    uploadFileName: "RobotoCondensed-Regular.ttf",
    licenseText: "Apache License Version 2.0",
  },
  {
    id: "roboto_slab",
    label: "Roboto Slab",
    family: "Roboto Slab",
    familyQuery: "Roboto+Slab:wght@400;700",
    fallback: "Georgia, serif",
    description: "Stronger serif rhythm for longer text.",
    uploadFileName: "RobotoSlab-Regular.ttf",
    licenseText: "Apache License Version 2.0",
  },
];

const BUILT_IN_PHRASES = [
  {
    id: "hero",
    label: "Launch line",
    note: "Hero message",
    text: "Pick a font. Break it on purpose. See if it survives.",
  },
  {
    id: "checkout",
    label: "Dashboard numerals",
    note: "Numbers, dates, and slashes",
    text: "Order 1001 ships to 10 I/O labs on 05/18 at 8:15 PM.",
  },
  {
    id: "confusion",
    label: "Confusion pairs",
    note: "Shape collisions",
    text: "O0 I l 1 S5 B8 rn m cl d",
  },
  {
    id: "dense",
    label: "Dense mobile copy",
    note: "Small interface reading",
    text: "Returns, refunds, and receipts should stay legible when contrast falls, spacing tightens, and the screen gets less forgiving.",
  },
];

const SCENARIOS = {
  web_light_default: {
    title: "Web light default",
    description: "Balanced reading on a bright web surface.",
    note: "Baseline approval state.",
    mode: "light",
  },
  mobile_dark_low_contrast: {
    title: "Mobile dark low contrast",
    description: "Smaller text on a darker, lower-contrast UI.",
    note: "Closer to the state where trust starts to slip.",
    mode: "night",
  },
};

const PRESETS = {
  balanced: {
    displaySize: "clamp(2rem, 3vw, 3.05rem)",
    letterSpacing: "-0.05em",
    filter: "none",
    opacity: "1",
    scaleX: "1",
  },
  contrast_loss: {
    displaySize: "clamp(1.92rem, 2.8vw, 2.7rem)",
    letterSpacing: "-0.045em",
    filter: "none",
    opacity: "0.74",
    scaleX: "1",
  },
  compression: {
    displaySize: "clamp(1.84rem, 2.7vw, 2.55rem)",
    letterSpacing: "-0.055em",
    filter: "none",
    opacity: "0.92",
    scaleX: "0.97",
  },
  blur: {
    displaySize: "clamp(1.88rem, 2.7vw, 2.58rem)",
    letterSpacing: "-0.048em",
    filter: "blur(0.45px)",
    opacity: "0.88",
    scaleX: "1",
  },
};

const STRESS_CONDITIONS = [
  {
    id: "reference",
    title: "Reference",
    summary: "Web light",
    scenario: "web_light_default",
    preset: "balanced",
    note: "Clean baseline.",
  },
  {
    id: "contrast_drop",
    title: "Contrast drop",
    summary: "Lower contrast",
    scenario: "web_light_default",
    preset: "contrast_loss",
    note: "Edges weaken first.",
  },
  {
    id: "compression",
    title: "Compression",
    summary: "Dense UI",
    scenario: "web_light_default",
    preset: "compression",
    note: "Spacing gets tighter.",
  },
  {
    id: "mobile_dark",
    title: "Mobile dark",
    summary: "Small and dim",
    scenario: "mobile_dark_low_contrast",
    preset: "blur",
    note: "Dark, softer, less forgiving.",
  },
];

const SCORE_NARRATIVES = [
  { limit: 0.85, text: "Strong result. The font stays steady in the selected profile." },
  { limit: 0.65, text: "Good overall, but pressure is starting to show." },
  { limit: 0.4, text: "Mixed result. Expect visible weakness in this setting." },
  { limit: 0, text: "Fragile result. This environment exposes real readability problems." },
];

const BENCH_CORPUS =
  "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()-_=+[]{};:'\\\",.<>/?\\\\|`~ O/0 I/l/1 S/5 B/8 rn/m cl/d";

const starterGrid = document.getElementById("starterGrid");
const customPhraseInput = document.getElementById("customPhraseInput");
const sessionSummary = document.getElementById("sessionSummary");
const conditionLegend = document.getElementById("conditionLegend");
const phraseBoard = document.getElementById("phraseBoard");
const reportProfileGrid = document.getElementById("reportProfileGrid");
const uploadTitle = document.getElementById("uploadTitle");
const fileInput = document.getElementById("fileInput");
const uploadBtn = document.getElementById("uploadBtn");
const uploadMsg = document.getElementById("uploadMsg");
const licenseSelect = document.getElementById("licenseSelect");
const generateReportBtn = document.getElementById("generateReportBtn");
const runStatus = document.getElementById("runStatus");
const registryList = document.getElementById("registryList");
const refreshRegistryBtn = document.getElementById("refreshRegistryBtn");
const reportEmpty = document.getElementById("reportEmpty");
const reportView = document.getElementById("reportView");
const metricGrid = document.getElementById("metricGrid");
const reportStory = document.getElementById("reportStory");
const reportJson = document.getElementById("reportJson");

const state = {
  selectedStarterId: STARTER_FONTS[0].id,
  selectedReportProfile: "web_light_default",
  registry: [],
  selectedSource: {
    type: "starter",
    starterId: STARTER_FONTS[0].id,
    label: STARTER_FONTS[0].label,
    previewFont: `"${STARTER_FONTS[0].family}", ${STARTER_FONTS[0].fallback}`,
    detail: "Google Fonts starter",
    asset: null,
  },
  pendingUpload: null,
  uploadPreviewUrl: null,
  uploadStyleNode: null,
  lastReport: null,
  lastReportProfile: null,
};

function escapeHtml(text) {
  return String(text ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll("\"", "&quot;")
    .replaceAll("'", "&#39;");
}

function titleCaseStatus(status) {
  return status.charAt(0).toUpperCase() + status.slice(1);
}

function classifyScore(score) {
  return SCORE_NARRATIVES.find((entry) => score >= entry.limit).text;
}

function findStarter(starterId) {
  return STARTER_FONTS.find((starter) => starter.id === starterId);
}

function currentPhrases() {
  const phrases = BUILT_IN_PHRASES.slice();
  const custom = customPhraseInput.value.trim();
  if (custom) {
    phrases.unshift({
      id: "custom",
      label: "Custom",
      note: "Your extra line",
      text: custom,
    });
  }
  return phrases;
}

function starterStylesheetId(starter) {
  return `starter-style-${starter.id}`;
}

async function ensureStarterFontLoaded(starter) {
  let link = document.getElementById(starterStylesheetId(starter));
  if (!link) {
    link = document.createElement("link");
    link.id = starterStylesheetId(starter);
    link.rel = "stylesheet";
    link.href =
      `https://fonts.googleapis.com/css2?family=${starter.familyQuery}` +
      `&display=swap&text=${encodeURIComponent(BENCH_CORPUS)}`;
    document.head.appendChild(link);
    await new Promise((resolve) => {
      link.addEventListener("load", resolve, { once: true });
      link.addEventListener("error", resolve, { once: true });
      window.setTimeout(resolve, 1200);
    });
  }

  if (document.fonts && document.fonts.load) {
    try {
      await Promise.all([
        document.fonts.load(`400 24px "${starter.family}"`),
        document.fonts.load(`700 32px "${starter.family}"`),
      ]);
    } catch (_error) {
      // Best effort only.
    }
  }
}

async function fetchStarterFontBlob(starter) {
  const cssUrl =
    `https://fonts.googleapis.com/css2?family=${starter.familyQuery}` +
    `&display=swap&text=${encodeURIComponent(BENCH_CORPUS)}`;
  const cssResponse = await fetch(cssUrl);
  if (!cssResponse.ok) {
    throw new Error(`failed to load starter CSS: ${cssResponse.status}`);
  }
  const css = await cssResponse.text();
  const match = css.match(/src:\s*url\(([^)]+)\)/);
  if (!match) {
    throw new Error("could not find starter font file URL");
  }
  const fontResponse = await fetch(match[1]);
  if (!fontResponse.ok) {
    throw new Error(`failed to download starter font: ${fontResponse.status}`);
  }
  return fontResponse.blob();
}

function findRegistryMatchByFileName(fileName) {
  return state.registry.find((asset) => asset.file_name === fileName) || null;
}

function clearReport() {
  state.lastReport = null;
  state.lastReportProfile = null;
  reportEmpty.hidden = false;
  reportView.hidden = true;
  metricGrid.innerHTML = "";
  reportStory.innerHTML = "";
  reportJson.textContent = "";
}

function setSelectedSource(nextSource, statusMessage) {
  state.selectedSource = nextSource;
  clearReport();
  runStatus.textContent = statusMessage;
}

function sampleFontSize(text, condition) {
  const preset = PRESETS[condition.preset];
  const length = text.length;
  if (length > 120) {
    return "clamp(0.98rem, 1.35vw, 1.18rem)";
  }
  if (length > 90) {
    return "clamp(1.08rem, 1.55vw, 1.34rem)";
  }
  if (length > 64) {
    return "clamp(1.22rem, 1.9vw, 1.6rem)";
  }
  if (length > 40) {
    return "clamp(1.45rem, 2.35vw, 1.95rem)";
  }
  return preset.displaySize;
}

function sampleStyle(text, condition) {
  const preset = PRESETS[condition.preset];
  const previewFont = state.selectedSource.previewFont || "system-ui, sans-serif";
  return [
    `font-family:${previewFont}`,
    `font-size:${sampleFontSize(text, condition)}`,
    `letter-spacing:${preset.letterSpacing}`,
    `opacity:${preset.opacity}`,
    `filter:${preset.filter}`,
    `transform:scaleX(${preset.scaleX})`,
  ].join(";");
}

function renderStarterGrid() {
  starterGrid.innerHTML = STARTER_FONTS.map((starter) => {
    const selected =
      (state.selectedSource.type === "starter" && state.selectedSource.starterId === starter.id) ||
      (state.selectedSource.asset && state.selectedSource.asset.file_name === starter.uploadFileName);
    return `
      <button class="starter-card ${selected ? "is-selected" : ""}" data-starter="${starter.id}">
        <div class="starter-swatch" style="font-family:'${starter.family}', ${starter.fallback};">
          <span class="starter-name">${starter.label}</span>
          <p>${starter.description}</p>
        </div>
        <span class="source-tag">Google Fonts</span>
      </button>
    `;
  }).join("");

  starterGrid.querySelectorAll("[data-starter]").forEach((card) => {
    card.addEventListener("click", async () => {
      const starter = findStarter(card.dataset.starter);
      state.selectedStarterId = starter.id;
      await ensureStarterFontLoaded(starter);
      setSelectedSource(
        {
          type: "starter",
          starterId: starter.id,
          label: starter.label,
          previewFont: `"${starter.family}", ${starter.fallback}`,
          detail: "Google Fonts starter",
          asset: findRegistryMatchByFileName(starter.uploadFileName),
        },
        `Showing ${starter.label} across the full stress board.`
      );
      renderAll();
    });
  });
}

function renderSessionSummary() {
  const phrases = currentPhrases();
  sessionSummary.innerHTML = `
    <div class="session-item">
      <strong>Font</strong>
      <span>${escapeHtml(state.selectedSource.label)}</span>
    </div>
    <div class="session-item">
      <strong>Source</strong>
      <span>${escapeHtml(state.selectedSource.detail)}</span>
    </div>
    <div class="session-item">
      <strong>Phrases</strong>
      <span>${phrases.length} visible</span>
    </div>
    <div class="session-item">
      <strong>Conditions</strong>
      <span>${STRESS_CONDITIONS.length} always on</span>
    </div>
  `;
}

function renderConditionLegend() {
  conditionLegend.innerHTML = STRESS_CONDITIONS.map((condition) => `
    <article class="legend-card">
      <strong>${condition.title}</strong>
      <span>${condition.summary}</span>
    </article>
  `).join("");
}

function renderPhraseBoard() {
  phraseBoard.innerHTML = currentPhrases().map((phrase) => `
    <section class="phrase-row">
      <div class="phrase-head">
        <div class="phrase-copy">
          <span class="phrase-tag">${escapeHtml(phrase.label)}</span>
          <p>${escapeHtml(phrase.note)}</p>
        </div>
      </div>
      <div class="specimen-grid">
        ${STRESS_CONDITIONS.map((condition) => `
          <article class="specimen-card ${condition.id}">
            <div class="specimen-meta">
              <strong>${condition.title}</strong>
              <span>${condition.summary}</span>
            </div>
            <p class="specimen-display" style="${sampleStyle(phrase.text, condition)}">${escapeHtml(phrase.text)}</p>
            <p class="specimen-note">${condition.note}</p>
          </article>
        `).join("")}
      </div>
    </section>
  `).join("");
}

function renderReportProfileGrid() {
  reportProfileGrid.innerHTML = Object.entries(SCENARIOS).map(([id, scenario]) => `
    <button class="profile-card ${state.selectedReportProfile === id ? "is-selected" : ""}" data-report-profile="${id}">
      <strong>${scenario.title}</strong>
      <p>${scenario.description}</p>
    </button>
  `).join("");

  reportProfileGrid.querySelectorAll("[data-report-profile]").forEach((button) => {
    button.addEventListener("click", () => {
      state.selectedReportProfile = button.dataset.reportProfile;
      clearReport();
      runStatus.textContent = `Report profile set to ${SCENARIOS[state.selectedReportProfile].title}.`;
      renderAll();
    });
  });
}

function renderRegistry() {
  if (!state.registry.length) {
    registryList.innerHTML = `<div class="empty-state">Registry is empty. Start with a starter font or upload one.</div>`;
    return;
  }

  registryList.innerHTML = state.registry.map((asset) => {
    const selectedAssetId = state.selectedSource.asset ? state.selectedSource.asset.id : "";
    const isSelected = selectedAssetId === asset.id;
    const title = escapeHtml(asset.family_name || asset.file_name);
    const fileName = escapeHtml(asset.file_name);
    return `
      <article class="registry-card ${isSelected ? "is-selected" : ""}">
        <div>
          <div class="registry-title">${title}</div>
          <p class="registry-meta">${fileName}</p>
          <p class="registry-meta">${escapeHtml(asset.license_normalized)} · ${escapeHtml(asset.status)}</p>
        </div>
        <div class="registry-actions">
          <span class="status-pill ${asset.status}">${titleCaseStatus(asset.status)}</span>
          <button class="secondary-button" data-use-asset="${asset.id}">Use</button>
        </div>
      </article>
    `;
  }).join("");

  registryList.querySelectorAll("[data-use-asset]").forEach((button) => {
    button.addEventListener("click", () => {
      const asset = state.registry.find((entry) => entry.id === button.dataset.useAsset);
      if (!asset) {
        return;
      }

      const starter = STARTER_FONTS.find((entry) => entry.uploadFileName === asset.file_name);
      setSelectedSource(
        {
          type: "registry",
          label: asset.family_name || asset.file_name,
          detail: "Existing registry asset",
          asset,
          previewFont: starter ? `"${starter.family}", ${starter.fallback}` : "system-ui, sans-serif",
        },
        `Showing ${asset.family_name || asset.file_name} from the registry.`
      );

      if (starter) {
        state.selectedStarterId = starter.id;
        ensureStarterFontLoaded(starter).catch(() => {});
      }

      renderAll();
    });
  });
}

function renderReport(report) {
  const scorePercent = Math.round(report.measurements.score * 100);
  const coveragePercent = Math.round(report.measurements.estimated_coverage * 100);
  metricGrid.innerHTML = `
    <article class="metric-card">
      <strong>${scorePercent}</strong>
      <span>Score</span>
    </article>
    <article class="metric-card">
      <strong>${coveragePercent}</strong>
      <span>Estimated coverage</span>
    </article>
    <article class="metric-card">
      <strong>${report.measurements.line_density.toFixed(1)}</strong>
      <span>Line density</span>
    </article>
  `;

  reportStory.innerHTML = `
    <h3>${escapeHtml(report.font.family_name || report.font.font_id)}</h3>
    <p>${escapeHtml(classifyScore(report.measurements.score))}</p>
    <p>${escapeHtml(report.measurements.notes)}</p>
  `;

  reportJson.textContent = JSON.stringify(report, null, 2);
  reportEmpty.hidden = true;
  reportView.hidden = false;
}

function renderAll() {
  renderStarterGrid();
  renderSessionSummary();
  renderConditionLegend();
  renderPhraseBoard();
  renderReportProfileGrid();
  renderRegistry();

  if (state.lastReport) {
    renderReport(state.lastReport);
  }
}

async function refreshRegistry() {
  registryList.innerHTML = `<div class="empty-state">Refreshing registry...</div>`;
  try {
    const response = await fetch("/api/fonts");
    const payload = await response.json();
    state.registry = Array.isArray(payload.assets) ? payload.assets : [];
    state.registry.sort((left, right) => {
      const statusRank = { approved: 0, quarantined: 1, rejected: 2 };
      const rankDiff = (statusRank[left.status] || 9) - (statusRank[right.status] || 9);
      if (rankDiff !== 0) {
        return rankDiff;
      }
      return (left.family_name || left.file_name).localeCompare(right.family_name || right.file_name);
    });
  } catch (error) {
    registryList.innerHTML = `<div class="empty-state">Failed to load registry: ${error.message}</div>`;
    return;
  }

  if (state.selectedSource.type === "starter") {
    const starter = findStarter(state.selectedSource.starterId);
    state.selectedSource.asset = findRegistryMatchByFileName(starter.uploadFileName);
  }

  if (state.selectedSource.type === "upload" && state.pendingUpload) {
    state.selectedSource.asset = findRegistryMatchByFileName(state.pendingUpload.name);
  }

  if (state.selectedSource.type === "registry" && state.selectedSource.asset) {
    state.selectedSource.asset =
      state.registry.find((asset) => asset.id === state.selectedSource.asset.id) || state.selectedSource.asset;
  }

  renderRegistry();
}

async function uploadPendingFile() {
  if (!state.pendingUpload) {
    throw new Error("no local file selected");
  }

  const formData = new FormData();
  formData.append("file", state.pendingUpload, state.pendingUpload.name);

  if (licenseSelect.value !== "unknown") {
    const sidecarName = state.pendingUpload.name.replace(/\.[^.]+$/, "") + ".license";
    formData.append(
      "file",
      new Blob([licenseSelect.value], { type: "text/plain" }),
      sidecarName
    );
  }

  const response = await fetch("/api/fonts/ingest", { method: "POST", body: formData });
  const payload = await response.json();
  if (!response.ok) {
    throw new Error(payload.error || "upload failed");
  }

  uploadMsg.textContent = `Uploaded ${state.pendingUpload.name}. Registry now has ${payload.total} item(s).`;
  await refreshRegistry();
  const asset = findRegistryMatchByFileName(state.pendingUpload.name);
  if (!asset) {
    throw new Error("upload completed but registry entry was not found");
  }

  setSelectedSource(
    {
      type: "upload",
      label: state.pendingUpload.name,
      detail: "Uploaded local file",
      previewFont: state.selectedSource.previewFont,
      asset,
    },
    `Uploaded ${state.pendingUpload.name} and switched the board to it.`
  );

  renderAll();
  return asset;
}

async function ensureBenchAsset() {
  if (state.selectedSource.asset) {
    return state.selectedSource.asset;
  }

  if (state.selectedSource.type === "starter") {
    const starter = findStarter(state.selectedSource.starterId);
    runStatus.textContent = `Syncing ${starter.label} into the registry...`;
    const blob = await fetchStarterFontBlob(starter);
    const formData = new FormData();
    formData.append("file", blob, starter.uploadFileName);
    formData.append(
      "file",
      new Blob([starter.licenseText], { type: "text/plain" }),
      starter.uploadFileName.replace(/\.[^.]+$/, ".license")
    );

    const response = await fetch("/api/fonts/ingest", { method: "POST", body: formData });
    const payload = await response.json();
    if (!response.ok) {
      throw new Error(payload.error || "starter sync failed");
    }

    await refreshRegistry();
    const asset = findRegistryMatchByFileName(starter.uploadFileName);
    if (!asset) {
      throw new Error("starter synced but registry entry was not found");
    }
    state.selectedSource.asset = asset;
    return asset;
  }

  if (state.selectedSource.type === "upload") {
    return uploadPendingFile();
  }

  throw new Error("choose a starter font or upload a file first");
}

async function runReport() {
  generateReportBtn.disabled = true;
  try {
    const asset = await ensureBenchAsset();
    runStatus.textContent = `Generating ${SCENARIOS[state.selectedReportProfile].title} report for ${asset.family_name || asset.file_name}...`;
    const response = await fetch(`/api/fonts/${asset.id}/report?profile=${state.selectedReportProfile}`);
    const payload = await response.json();
    if (!response.ok) {
      throw new Error(payload.error || "report failed");
    }
    state.lastReport = payload;
    state.lastReportProfile = state.selectedReportProfile;
    renderReport(payload);
    runStatus.textContent = `Report ready for ${asset.family_name || asset.file_name}.`;
  } catch (error) {
    runStatus.textContent = `Report failed: ${error.message}`;
  } finally {
    generateReportBtn.disabled = false;
  }
}

function updateUploadPreview(file) {
  uploadTitle.textContent = file.name;
  uploadBtn.disabled = false;
  uploadMsg.textContent = "Previewing the local file. Upload it only if you want it in the registry.";

  if (state.uploadPreviewUrl) {
    URL.revokeObjectURL(state.uploadPreviewUrl);
  }

  state.uploadPreviewUrl = URL.createObjectURL(file);
  const family = `TypeWeaverLocal-${Date.now()}`;
  if (state.uploadStyleNode) {
    state.uploadStyleNode.remove();
  }

  state.uploadStyleNode = document.createElement("style");
  state.uploadStyleNode.textContent = `
    @font-face {
      font-family: "${family}";
      src: url("${state.uploadPreviewUrl}");
    }
  `;
  document.head.appendChild(state.uploadStyleNode);

  state.pendingUpload = file;
  setSelectedSource(
    {
      type: "upload",
      label: file.name,
      detail: "Local preview, not yet uploaded",
      previewFont: `"${family}", system-ui, sans-serif`,
      asset: findRegistryMatchByFileName(file.name),
    },
    `Previewing ${file.name} across the full stress board.`
  );
  renderAll();
}

async function initialize() {
  await ensureStarterFontLoaded(STARTER_FONTS[0]);
  renderAll();
  await refreshRegistry();
  runStatus.textContent = "Ready. Pick a font or keep the default.";
}

customPhraseInput.addEventListener("input", () => {
  runStatus.textContent = customPhraseInput.value.trim()
    ? "Custom phrase added to the board."
    : "Custom phrase removed. Built-ins remain.";
  renderAll();
});

fileInput.addEventListener("change", () => {
  const [file] = fileInput.files;
  if (!file) {
    return;
  }
  updateUploadPreview(file);
});

uploadBtn.addEventListener("click", async () => {
  uploadBtn.disabled = true;
  try {
    await uploadPendingFile();
  } catch (error) {
    uploadMsg.textContent = `Upload failed: ${error.message}`;
  } finally {
    uploadBtn.disabled = !state.pendingUpload;
  }
});

generateReportBtn.addEventListener("click", runReport);
refreshRegistryBtn.addEventListener("click", refreshRegistry);

initialize().catch((error) => {
  runStatus.textContent = `Startup failed: ${error.message}`;
});
