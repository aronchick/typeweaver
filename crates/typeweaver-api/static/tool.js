const STARTER_FONTS = [
  {
    id: "roboto",
    label: "Roboto",
    family: "Roboto",
    familyQuery: "Roboto:wght@400;700",
    fallback: "system-ui, sans-serif",
    description: "Stable default for UI and product copy.",
    uploadFileName: "Roboto-Regular.ttf",
    licenseText: "Apache License Version 2.0",
  },
  {
    id: "roboto_condensed",
    label: "Roboto Condensed",
    family: "Roboto Condensed",
    familyQuery: "Roboto+Condensed:wght@400;700",
    fallback: "\"Arial Narrow\", sans-serif",
    description: "A tighter read for denser navigation and tables.",
    uploadFileName: "RobotoCondensed-Regular.ttf",
    licenseText: "Apache License Version 2.0",
  },
  {
    id: "roboto_slab",
    label: "Roboto Slab",
    family: "Roboto Slab",
    familyQuery: "Roboto+Slab:wght@400;700",
    fallback: "Georgia, serif",
    description: "Editorial texture with stronger serif rhythm.",
    uploadFileName: "RobotoSlab-Regular.ttf",
    licenseText: "Apache License Version 2.0",
  },
];

const PHRASES = [
  {
    id: "hero",
    label: "Launch message",
    text: "Pick a font. Break it on purpose. See if it survives.",
  },
  {
    id: "checkout",
    label: "Dashboard numerals",
    text: "Order 1001 ships to 10 I/O labs on 05/18 at 8:15 PM.",
  },
  {
    id: "confusion",
    label: "Confusion pairs",
    text: "O0 I l 1 S5 B8 rn m cl d",
  },
  {
    id: "dense",
    label: "Dense mobile copy",
    text: "Returns, refunds, and receipts should stay legible when contrast falls, spacing tightens, and the screen gets less forgiving.",
  },
];

const SCENARIOS = {
  web_light_default: {
    title: "Web light default",
    description: "Balanced desktop or laptop reading with normal contrast.",
    note: "This is the safer baseline where many fonts look fine before the harder states show up.",
    mode: "light",
  },
  mobile_dark_low_contrast: {
    title: "Mobile dark low contrast",
    description: "Smaller text, darker UI, and reduced contrast tolerance.",
    note: "This profile gets closer to the point where a font stops feeling trustworthy.",
    mode: "night",
  },
};

const PRESETS = {
  balanced: {
    title: "Balanced",
    description: "Minimal stress. Keep the specimen mostly intact.",
    summary: "Low stress",
    displaySize: "clamp(2rem, 3.4vw, 3.5rem)",
    bodySize: "16px",
    detailSize: "13px",
    letterSpacing: "-0.03em",
    filter: "none",
    opacity: "1",
    bodyOpacity: "0.88",
    scaleX: "1",
  },
  contrast_loss: {
    title: "Contrast loss",
    description: "Fade the text just enough to expose weak structure.",
    summary: "Lower contrast",
    displaySize: "clamp(1.95rem, 3.2vw, 3.15rem)",
    bodySize: "15px",
    detailSize: "12px",
    letterSpacing: "-0.035em",
    filter: "none",
    opacity: "0.7",
    bodyOpacity: "0.68",
    scaleX: "1",
  },
  compression: {
    title: "Compression",
    description: "Tighter measure, smaller size, slightly squeezed width.",
    summary: "Dense UI",
    displaySize: "clamp(1.85rem, 3vw, 3rem)",
    bodySize: "14px",
    detailSize: "12px",
    letterSpacing: "-0.045em",
    filter: "none",
    opacity: "0.94",
    bodyOpacity: "0.82",
    scaleX: "0.97",
  },
  blur: {
    title: "Blur + glow",
    description: "A little softness to mimic a less forgiving screen.",
    summary: "Soft focus",
    displaySize: "clamp(1.9rem, 3.1vw, 3.05rem)",
    bodySize: "14px",
    detailSize: "12px",
    letterSpacing: "-0.04em",
    filter: "blur(0.45px)",
    opacity: "0.88",
    bodyOpacity: "0.74",
    scaleX: "1",
  },
};

const SCORE_NARRATIVES = [
  { limit: 0.85, text: "Strong result. The font stays steady in the selected scenario." },
  { limit: 0.65, text: "Good overall, but pressure is starting to show." },
  { limit: 0.4, text: "Mixed result. Expect visible weakness in this setting." },
  { limit: 0, text: "Fragile result. This environment exposes real readability problems." },
];

const BENCH_CORPUS =
  "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()-_=+[]{};:'\\\",.<>/?\\\\|`~ O/0 I/l/1 S/5 B/8 rn/m cl/d";

const starterGrid = document.getElementById("starterGrid");
const phraseGrid = document.getElementById("phraseGrid");
const phraseInput = document.getElementById("phraseInput");
const scenarioGrid = document.getElementById("scenarioGrid");
const presetGrid = document.getElementById("presetGrid");
const sessionSummary = document.getElementById("sessionSummary");
const uploadTitle = document.getElementById("uploadTitle");
const fileInput = document.getElementById("fileInput");
const uploadBtn = document.getElementById("uploadBtn");
const uploadMsg = document.getElementById("uploadMsg");
const licenseSelect = document.getElementById("licenseSelect");
const runPreviewBtn = document.getElementById("runPreviewBtn");
const generateReportBtn = document.getElementById("generateReportBtn");
const runStatus = document.getElementById("runStatus");
const referenceCard = document.getElementById("referenceCard");
const stressedCard = document.getElementById("stressedCard");
const referenceMeta = document.getElementById("referenceMeta");
const stressedMeta = document.getElementById("stressedMeta");
const referenceDisplay = document.getElementById("referenceDisplay");
const stressedDisplay = document.getElementById("stressedDisplay");
const referenceBody = document.getElementById("referenceBody");
const stressedBody = document.getElementById("stressedBody");
const referenceDetail = document.getElementById("referenceDetail");
const stressedDetail = document.getElementById("stressedDetail");
const matrixGrid = document.getElementById("matrixGrid");
const registryList = document.getElementById("registryList");
const refreshRegistryBtn = document.getElementById("refreshRegistryBtn");
const reportEmpty = document.getElementById("reportEmpty");
const reportView = document.getElementById("reportView");
const metricGrid = document.getElementById("metricGrid");
const reportStory = document.getElementById("reportStory");
const reportJson = document.getElementById("reportJson");

const state = {
  selectedStarterId: STARTER_FONTS[0].id,
  selectedPhraseId: PHRASES[0].id,
  selectedScenario: "web_light_default",
  selectedPreset: "balanced",
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
  uploadPreviewName: null,
  uploadPreviewUrl: null,
  uploadStyleNode: null,
  lastReport: null,
};

function titleCaseStatus(status) {
  return status.charAt(0).toUpperCase() + status.slice(1);
}

function classifyScore(score) {
  return SCORE_NARRATIVES.find((entry) => score >= entry.limit).text;
}

function findStarter(starterId) {
  return STARTER_FONTS.find((starter) => starter.id === starterId);
}

function currentPhraseText() {
  return phraseInput.value.trim() || PHRASES.find((item) => item.id === state.selectedPhraseId).text;
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
      state.selectedSource = {
        type: "starter",
        starterId: starter.id,
        label: starter.label,
        previewFont: `"${starter.family}", ${starter.fallback}`,
        detail: "Google Fonts starter",
        asset: findRegistryMatchByFileName(starter.uploadFileName) || null,
      };
      renderAll();
    });
  });
}

function renderPhraseGrid() {
  phraseGrid.innerHTML = PHRASES.map((phrase) => `
    <button class="option-card ${state.selectedPhraseId === phrase.id ? "is-selected" : ""}" data-phrase="${phrase.id}">
      <strong>${phrase.label}</strong>
      <p>${phrase.text}</p>
    </button>
  `).join("");

  phraseGrid.querySelectorAll("[data-phrase]").forEach((card) => {
    card.addEventListener("click", () => {
      const phrase = PHRASES.find((item) => item.id === card.dataset.phrase);
      state.selectedPhraseId = phrase.id;
      phraseInput.value = phrase.text;
      renderAll();
    });
  });
}

function renderScenarioGrid() {
  scenarioGrid.innerHTML = Object.entries(SCENARIOS).map(([id, scenario]) => `
    <button class="option-card ${state.selectedScenario === id ? "is-selected" : ""}" data-scenario="${id}">
      <strong>${scenario.title}</strong>
      <p>${scenario.description}</p>
    </button>
  `).join("");

  scenarioGrid.querySelectorAll("[data-scenario]").forEach((card) => {
    card.addEventListener("click", () => {
      state.selectedScenario = card.dataset.scenario;
      renderAll();
    });
  });
}

function renderPresetGrid() {
  presetGrid.innerHTML = Object.entries(PRESETS).map(([id, preset]) => `
    <button class="option-card ${state.selectedPreset === id ? "is-selected" : ""}" data-preset="${id}">
      <strong>${preset.title}</strong>
      <p>${preset.description}</p>
    </button>
  `).join("");

  presetGrid.querySelectorAll("[data-preset]").forEach((card) => {
    card.addEventListener("click", () => {
      state.selectedPreset = card.dataset.preset;
      renderAll();
    });
  });
}

function renderSessionSummary() {
  const scenario = SCENARIOS[state.selectedScenario];
  const preset = PRESETS[state.selectedPreset];
  sessionSummary.innerHTML = `
    <div class="session-item">
      <strong>Source</strong>
      <span>${state.selectedSource.label}</span>
    </div>
    <div class="session-item">
      <strong>Path</strong>
      <span>${state.selectedSource.detail}</span>
    </div>
    <div class="session-item">
      <strong>Scenario</strong>
      <span>${scenario.title}</span>
    </div>
    <div class="session-item">
      <strong>Preset</strong>
      <span>${preset.summary}</span>
    </div>
  `;
}

function applyScenarioClasses() {
  const isNight = SCENARIOS[state.selectedScenario].mode === "night";
  [referenceCard, stressedCard].forEach((card) => {
    card.classList.toggle("is-night", isNight);
  });
}

function applySurfaceStyles() {
  const scenario = SCENARIOS[state.selectedScenario];
  const preset = PRESETS[state.selectedPreset];
  const previewFont = state.selectedSource.previewFont || "system-ui, sans-serif";
  const phrase = currentPhraseText();
  const breakdown = `Scenario: ${scenario.title}. Preset: ${preset.title}.`;
  const detail = "Confusion pairs: O0 I/l/1 S5 B8 rn/m cl/d";

  referenceMeta.textContent = scenario.title;
  stressedMeta.textContent = `${scenario.title} + ${preset.title}`;
  referenceDisplay.textContent = phrase;
  stressedDisplay.textContent = phrase;
  referenceBody.textContent = scenario.note;
  stressedBody.textContent = `${preset.description} ${scenario.note}`;
  referenceDetail.textContent = detail;
  stressedDetail.textContent = `${detail} ${breakdown}`;

  [
    referenceDisplay,
    stressedDisplay,
    referenceBody,
    stressedBody,
    referenceDetail,
    stressedDetail,
  ].forEach((node) => {
    node.style.fontFamily = previewFont;
  });

  referenceDisplay.style.fontSize = "clamp(2.3rem, 4vw, 4rem)";
  referenceDisplay.style.letterSpacing = "-0.05em";
  referenceBody.style.fontSize = scenario.mode === "night" ? "15px" : "16px";
  referenceDetail.style.fontSize = "13px";
  referenceDisplay.style.opacity = "1";
  referenceBody.style.opacity = scenario.mode === "night" ? "0.84" : "0.88";
  referenceDetail.style.opacity = scenario.mode === "night" ? "0.72" : "0.84";
  referenceDisplay.style.transform = "scaleX(1)";
  referenceBody.style.transform = "scaleX(1)";
  referenceDetail.style.transform = "scaleX(1)";
  referenceDisplay.style.filter = "none";
  referenceBody.style.filter = "none";
  referenceDetail.style.filter = "none";

  stressedDisplay.style.fontSize = preset.displaySize;
  stressedBody.style.fontSize = preset.bodySize;
  stressedDetail.style.fontSize = preset.detailSize;
  stressedDisplay.style.letterSpacing = preset.letterSpacing;
  stressedBody.style.letterSpacing = preset.letterSpacing;
  stressedDetail.style.letterSpacing = preset.letterSpacing;
  stressedDisplay.style.opacity = preset.opacity;
  stressedBody.style.opacity = preset.bodyOpacity;
  stressedDetail.style.opacity = preset.bodyOpacity;
  stressedDisplay.style.transform = `scaleX(${preset.scaleX})`;
  stressedBody.style.transform = `scaleX(${preset.scaleX})`;
  stressedDetail.style.transform = `scaleX(${preset.scaleX})`;
  stressedDisplay.style.filter = preset.filter;
  stressedBody.style.filter = preset.filter;
  stressedDetail.style.filter = preset.filter;
}

function renderMatrix() {
  const preset = PRESETS[state.selectedPreset];
  const isNight = SCENARIOS[state.selectedScenario].mode === "night";
  const previewFont = state.selectedSource.previewFont || "system-ui, sans-serif";
  const cards = [
    {
      title: "Display",
      text: currentPhraseText(),
      note: "How the main message behaves.",
    },
    {
      title: "Dense body",
      text: "Returns, refunds, and receipts should remain calm under pressure.",
      note: "Longer reading when the interface gets tighter.",
    },
    {
      title: "Confusion pairs",
      text: "O0 I l 1 S5 B8 rn m cl d",
      note: "The quick trust test for detail-heavy UI text.",
    },
  ];

  matrixGrid.innerHTML = cards.map((card) => `
    <article class="matrix-card ${isNight ? "is-night" : ""}">
      <strong>${card.title}</strong>
      <p>${card.note}</p>
      <div class="matrix-sample" style="
        font-family:${previewFont};
        font-size:${preset.displaySize};
        letter-spacing:${preset.letterSpacing};
        opacity:${preset.opacity};
        filter:${preset.filter};
        transform:scaleX(${preset.scaleX});
      ">${card.text}</div>
    </article>
  `).join("");
}

function renderRegistry() {
  if (!state.registry.length) {
    registryList.innerHTML = `<div class="empty-state">Registry is empty. Start with a starter font or upload one.</div>`;
    return;
  }

  registryList.innerHTML = state.registry.map((asset) => {
    const selectedAssetId = state.selectedSource.asset ? state.selectedSource.asset.id : "";
    const isSelected = selectedAssetId === asset.id;
    return `
      <article class="registry-card ${isSelected ? "is-selected" : ""}">
        <div>
          <div class="registry-title">${asset.family_name || asset.file_name}</div>
          <p class="registry-meta">${asset.file_name}</p>
          <p class="registry-meta">${asset.license_normalized} · ${asset.status}</p>
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
      state.selectedSource = {
        type: "registry",
        label: asset.family_name || asset.file_name,
        detail: "Existing registry asset",
        asset,
        previewFont: starter ? `"${starter.family}", ${starter.fallback}` : "system-ui, sans-serif",
      };
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
    <h3>${report.font.family_name || report.font.font_id}</h3>
    <p>${classifyScore(report.measurements.score)}</p>
    <p>${report.measurements.notes}</p>
  `;

  reportJson.textContent = JSON.stringify(report, null, 2);
  reportEmpty.hidden = true;
  reportView.hidden = false;
}

function renderAll() {
  renderStarterGrid();
  renderPhraseGrid();
  renderScenarioGrid();
  renderPresetGrid();
  renderSessionSummary();
  applyScenarioClasses();
  applySurfaceStyles();
  renderMatrix();
  renderRegistry();
}

function findRegistryMatchByFileName(fileName) {
  return state.registry.find((asset) => asset.file_name === fileName) || null;
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
  state.selectedSource = {
    type: "upload",
    label: state.pendingUpload.name,
    detail: "Uploaded local file",
    previewFont: state.selectedSource.previewFont,
    asset,
  };
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
    runStatus.textContent = `Generating report for ${asset.family_name || asset.file_name}...`;
    const response = await fetch(`/api/fonts/${asset.id}/report?profile=${state.selectedScenario}`);
    const payload = await response.json();
    if (!response.ok) {
      throw new Error(payload.error || "report failed");
    }
    state.lastReport = payload;
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
  uploadMsg.textContent = "Previewing the local file. Upload when you want it in the registry.";

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
  state.selectedSource = {
    type: "upload",
    label: file.name,
    detail: "Local preview, not yet uploaded",
    previewFont: `"${family}", system-ui, sans-serif`,
    asset: findRegistryMatchByFileName(file.name),
  };
  renderAll();
}

async function initialize() {
  phraseInput.value = PHRASES[0].text;
  await ensureStarterFontLoaded(STARTER_FONTS[0]);
  renderAll();
  await refreshRegistry();
}

phraseInput.addEventListener("input", renderAll);

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

runPreviewBtn.addEventListener("click", () => {
  const scenario = SCENARIOS[state.selectedScenario];
  const preset = PRESETS[state.selectedPreset];
  runStatus.textContent =
    `Updated the playground for ${state.selectedSource.label} in ${scenario.title} with ${preset.title}.`;
  renderAll();
});

generateReportBtn.addEventListener("click", runReport);
refreshRegistryBtn.addEventListener("click", refreshRegistry);

initialize().catch((error) => {
  runStatus.textContent = `Startup failed: ${error.message}`;
});
