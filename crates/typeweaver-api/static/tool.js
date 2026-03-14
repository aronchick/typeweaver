const STARTER_FONTS = [
  {
    id: "roboto",
    label: "Roboto",
    family: "Roboto",
    familyQuery: "Roboto:wght@400;700",
    fallback: "system-ui, sans-serif",
    uploadFileName: "Roboto-Regular.ttf",
    licenseText: "Apache License Version 2.0",
    detail: "Built-in example",
  },
  {
    id: "roboto_condensed",
    label: "Roboto Condensed",
    family: "Roboto Condensed",
    familyQuery: "Roboto+Condensed:wght@400;700",
    fallback: "\"Arial Narrow\", sans-serif",
    uploadFileName: "RobotoCondensed-Regular.ttf",
    licenseText: "Apache License Version 2.0",
    detail: "Built-in example",
  },
  {
    id: "roboto_slab",
    label: "Roboto Slab",
    family: "Roboto Slab",
    familyQuery: "Roboto+Slab:wght@400;700",
    fallback: "Georgia, serif",
    uploadFileName: "RobotoSlab-Regular.ttf",
    licenseText: "Apache License Version 2.0",
    detail: "Built-in example",
  },
];

const REPORT_PROFILES = {
  web_light_default: {
    title: "Web light",
    detail: "Bright page with regular spacing.",
  },
  mobile_dark_low_contrast: {
    title: "Dark mobile",
    detail: "Dim screen with tighter spacing.",
  },
};

const SCORE_NARRATIVES = [
  { limit: 0.85, text: "Strong result. This font stays readable in the selected check." },
  { limit: 0.65, text: "Good overall. A few weak spots show up, but the font still holds together." },
  { limit: 0.4, text: "Mixed result. Some scenes are clearly harder to read." },
  { limit: 0, text: "Fragile result. This check exposes real reading problems." },
];

const BENCH_CORPUS =
  "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()-_=+[]{};:'\\\",.<>/?\\\\|`~ O/0 I/l/1 S/5 B/8 rn/m cl/d";

const starterGrid = document.getElementById("starterGrid");
const currentFontLabel = document.getElementById("currentFontLabel");
const summaryGrid = document.getElementById("summaryGrid");
const customPhraseInput = document.getElementById("customPhraseInput");
const evidenceWall = document.getElementById("evidenceWall");
const fileInput = document.getElementById("fileInput");
const uploadTitle = document.getElementById("uploadTitle");
const uploadBtn = document.getElementById("uploadBtn");
const uploadMsg = document.getElementById("uploadMsg");
const fontUrlInput = document.getElementById("fontUrlInput");
const loadUrlBtn = document.getElementById("loadUrlBtn");
const urlMsg = document.getElementById("urlMsg");
const licenseSelect = document.getElementById("licenseSelect");
const generateReportBtn = document.getElementById("generateReportBtn");
const runStatus = document.getElementById("runStatus");
const reportProfileGrid = document.getElementById("reportProfileGrid");
const reportEmpty = document.getElementById("reportEmpty");
const reportView = document.getElementById("reportView");
const metricGrid = document.getElementById("metricGrid");
const reportStory = document.getElementById("reportStory");
const reportJson = document.getElementById("reportJson");
const registryList = document.getElementById("registryList");
const refreshRegistryBtn = document.getElementById("refreshRegistryBtn");

const state = {
  selectedStarterId: STARTER_FONTS[0].id,
  selectedReportProfile: "web_light_default",
  registry: [],
  selectedSource: {
    type: "starter",
    starterId: STARTER_FONTS[0].id,
    label: STARTER_FONTS[0].label,
    previewFont: `"${STARTER_FONTS[0].family}", ${STARTER_FONTS[0].fallback}`,
    detail: STARTER_FONTS[0].detail,
    asset: null,
  },
  pendingUpload: null,
  uploadPreviewUrl: null,
  uploadStyleNode: null,
  registryStyleNode: null,
  registryPreviewAssetId: null,
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

function starterStylesheetId(starter) {
  return `starter-style-${starter.id}`;
}

function currentCustomPhrase() {
  return customPhraseInput.value.trim();
}

function applyPageFont(previewFont) {
  document.documentElement.style.setProperty("--specimen-font", previewFont);
  currentFontLabel.textContent = state.selectedSource.label;
  document.title = `TypeWeaver | ${state.selectedSource.label}`;
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
  applyPageFont(nextSource.previewFont || "\"Space Grotesk\", \"Segoe UI\", sans-serif");
  clearReport();
  runStatus.textContent = statusMessage;
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
    throw new Error(`failed to load example font CSS: ${cssResponse.status}`);
  }

  const css = await cssResponse.text();
  const match = css.match(/src:\s*url\(([^)]+)\)/);
  if (!match) {
    throw new Error("could not find the example font file URL");
  }

  const fontResponse = await fetch(match[1]);
  if (!fontResponse.ok) {
    throw new Error(`failed to download example font: ${fontResponse.status}`);
  }

  return fontResponse.blob();
}

async function ensureRegistryAssetLoaded(asset) {
  const family = `TypeWeaverRegistry-${asset.id}`;
  if (state.registryPreviewAssetId !== asset.id) {
    if (state.registryStyleNode) {
      state.registryStyleNode.remove();
    }

    state.registryStyleNode = document.createElement("style");
    state.registryStyleNode.textContent = `
      @font-face {
        font-family: "${family}";
        src: url("/api/fonts/${asset.id}/file");
        font-display: swap;
      }
    `;
    document.head.appendChild(state.registryStyleNode);
    state.registryPreviewAssetId = asset.id;
  }

  if (document.fonts && document.fonts.load) {
    try {
      await Promise.all([
        document.fonts.load(`400 24px "${family}"`),
        document.fonts.load(`700 32px "${family}"`),
      ]);
    } catch (_error) {
      // Best effort only.
    }
  }

  return `"${family}", system-ui, sans-serif`;
}

function findRegistryMatchByFileName(fileName) {
  return state.registry.find((asset) => asset.file_name === fileName) || null;
}

function reportReadyLabel() {
  if (state.lastReport && state.lastReportProfile) {
    return `Last report: ${REPORT_PROFILES[state.lastReportProfile].title}`;
  }

  if (state.selectedSource.asset || state.selectedSource.type === "starter") {
    return `Ready for ${REPORT_PROFILES[state.selectedReportProfile].title}`;
  }

  if (state.pendingUpload) {
    return "Save the uploaded file before running a report";
  }

  return "Load a font to run a report";
}

function renderSummary() {
  summaryGrid.innerHTML = `
    <article class="summary-card">
      <span>Current font</span>
      <strong>${escapeHtml(state.selectedSource.label)}</strong>
      <p>${escapeHtml(state.selectedSource.detail)}</p>
    </article>
    <article class="summary-card">
      <span>Page state</span>
      <strong>Live specimen</strong>
      <p>This page is now set in the selected font.</p>
    </article>
    <article class="summary-card">
      <span>Report</span>
      <strong>${escapeHtml(REPORT_PROFILES[state.selectedReportProfile].title)}</strong>
      <p>${escapeHtml(reportReadyLabel())}</p>
    </article>
  `;
}

function renderStarterGrid() {
  starterGrid.innerHTML = STARTER_FONTS.map((starter) => {
    const selected =
      (state.selectedSource.type === "starter" && state.selectedSource.starterId === starter.id) ||
      (state.selectedSource.asset && state.selectedSource.asset.file_name === starter.uploadFileName);
    return `
      <button class="starter-card ${selected ? "is-selected" : ""}" data-starter="${starter.id}">
        <div class="starter-swatch" style="font-family:'${starter.family}', ${starter.fallback};">Aa 01 rn</div>
        <strong>${starter.label}</strong>
        <span>${starter.detail}</span>
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
          detail: starter.detail,
          asset: findRegistryMatchByFileName(starter.uploadFileName),
        },
        `Showing ${starter.label} across the page.`
      );
      renderAll();
    });
  });
}

function renderCheckCard({
  title,
  detail,
  leftLabel,
  rightLabel,
  leftClass,
  rightClass,
  leftContent,
  rightContent,
}) {
  return `
    <article class="check-card">
      <div class="check-head">
        <h3>${escapeHtml(title)}</h3>
        <p>${escapeHtml(detail)}</p>
      </div>
      <div class="comparison-grid">
        <section class="sample-panel ${leftClass}">
          <span class="card-kicker">${escapeHtml(leftLabel)}</span>
          ${leftContent}
        </section>
        <section class="sample-panel ${rightClass}">
          <span class="card-kicker">${escapeHtml(rightLabel)}</span>
          ${rightContent}
        </section>
      </div>
    </article>
  `;
}

function renderTinyCase() {
  const rows = [
    ["Order 1001", "8:15 PM"],
    ["Pass O0", "I l 1"],
    ["10 I/O labs", "Ready"],
  ];
  const buildRows = () =>
    rows
      .map(
        ([left, right]) => `
          <div class="mini-row">
            <span>${escapeHtml(left)}</span>
            <strong>${escapeHtml(right)}</strong>
          </div>
        `
      )
      .join("");

  return renderCheckCard({
    title: "Small labels",
    detail: "Short labels, times, and codes.",
    leftLabel: "Normal",
    rightLabel: "11px",
    leftClass: "is-paper",
    rightClass: "is-sand",
    leftContent: `<div class="mini-list">${buildRows()}</div>`,
    rightContent: `<div class="mini-list is-small">${buildRows()}</div>`,
  });
}

function renderContrastCase() {
  const text = "Order 1001 ships to 10 I/O labs.";
  return renderCheckCard({
    title: "Low contrast",
    detail: "Lighter text is often the first thing to disappear.",
    leftLabel: "Normal",
    rightLabel: "Lower contrast",
    leftClass: "is-paper",
    rightClass: "is-sand",
    leftContent: `
      <div class="sample-stack">
        <p class="sample-copy">${escapeHtml(text)}</p>
        <p class="sample-note">1001 · O0 · I l 1</p>
      </div>
    `,
    rightContent: `
      <div class="sample-stack">
        <p class="sample-copy is-fade">${escapeHtml(text)}</p>
        <p class="sample-note">1001 · O0 · I l 1</p>
      </div>
    `,
  });
}

function renderBlurCase() {
  const text = "Returns, refunds, receipts.";
  return renderCheckCard({
    title: "Soft edges",
    detail: "A little blur quickly changes how letters feel.",
    leftLabel: "Sharp",
    rightLabel: "Softer edges",
    leftClass: "is-paper",
    rightClass: "is-sand",
    leftContent: `
      <div class="sample-stack">
        <p class="sample-copy">${escapeHtml(text)}</p>
        <p class="sample-note">rn m · cl d · S5 · B8</p>
      </div>
    `,
    rightContent: `
      <div class="sample-stack">
        <p class="sample-copy is-blur">${escapeHtml(text)}</p>
        <p class="sample-note">rn m · cl d · S5 · B8</p>
      </div>
    `,
  });
}

function renderConfusableCase() {
  const pairs = ["I l 1", "O 0", "rn m", "cl d", "S 5", "B 8"];
  return renderCheckCard({
    title: "Look-alikes",
    detail: "Keep common confusion pairs apart.",
    leftLabel: "Spaced out",
    rightLabel: "Tighter together",
    leftClass: "is-paper",
    rightClass: "is-sand",
    leftContent: `
      <div class="pair-grid">
        ${pairs.map((pair) => `<div class="pair-cell">${escapeHtml(pair)}</div>`).join("")}
      </div>
    `,
    rightContent: `
      <div class="sample-stack">
        <p class="pair-line is-tight">Il1 O0 rnm cld S5 B8</p>
        <p class="sample-note">Dimmer and tighter</p>
      </div>
    `,
  });
}

function renderDenseCase() {
  const roomyRows = [
    ["Queue", "12"],
    ["Invoice 105", "Ready"],
    ["Gate C12", "10:31 PM"],
  ];
  const stressRows = [
    ["Queue 12", "Ready"],
    ["Invoice 105", "O0"],
    ["Returns", "I l 1"],
    ["Gate C12", "10:31 PM"],
    ["rn m", "cl d"],
  ];
  return renderCheckCard({
    title: "Packed lists",
    detail: "Tighter spacing exposes rhythm and shape problems.",
    leftLabel: "Roomy list",
    rightLabel: "Packed list",
    leftClass: "is-paper",
    rightClass: "is-sand",
    leftContent: `
      <div class="dense-board">
        ${roomyRows
          .map(
            ([left, right]) => `
              <div class="dense-row">
                <span>${escapeHtml(left)}</span>
                <span>${escapeHtml(right)}</span>
              </div>
            `
          )
          .join("")}
      </div>
    `,
    rightContent: `
      <div class="dense-board is-tight">
        ${stressRows
          .map(
            ([left, right]) => `
              <div class="dense-row">
                <span>${escapeHtml(left)}</span>
                <span>${escapeHtml(right)}</span>
              </div>
            `
          )
          .join("")}
        <div class="dense-row is-crowded">
          <span>Queue 12 / Invoice 105 / Gate C12 / O0 / rn m / 10:31 PM</span>
        </div>
      </div>
    `,
  });
}

function renderDarkCase() {
  const messages = [
    ["Returns", "Refunds, receipts, approvals."],
    ["Gate C12", "Boarding at 10:31 PM."],
    ["Order 1001", "10 I/O labs ready."],
  ];
  const buildMessages = () =>
    messages
      .map(
        ([title, text]) => `
          <div class="message-card">
            <strong>${escapeHtml(title)}</strong>
            <span>${escapeHtml(text)}</span>
          </div>
        `
      )
      .join("");

  return renderCheckCard({
    title: "Dark screen",
    detail: "A dim mobile view changes contrast and spacing at once.",
    leftLabel: "Bright screen",
    rightLabel: "Dark screen",
    leftClass: "is-paper",
    rightClass: "is-night",
    leftContent: `<div class="message-stack">${buildMessages()}</div>`,
    rightContent: `<div class="message-stack is-dark">${buildMessages()}</div>`,
  });
}

function renderCustomCase(text) {
  return renderCheckCard({
    title: "Your line",
    detail: "Read the exact sentence you care about.",
    leftLabel: "Normal",
    rightLabel: "Dimmer and tighter",
    leftClass: "is-paper",
    rightClass: "is-night",
    leftContent: `
      <div class="sample-stack">
        <p class="sample-copy">${escapeHtml(text)}</p>
        <p class="sample-note">Your own line</p>
      </div>
    `,
    rightContent: `
      <div class="sample-stack">
        <p class="sample-copy is-fade is-tight">${escapeHtml(text)}</p>
        <p class="sample-note">Low light and less room</p>
      </div>
    `,
  });
}

function renderEvidenceWall() {
  const cards = [
    renderTinyCase(),
    renderContrastCase(),
    renderBlurCase(),
    renderConfusableCase(),
    renderDenseCase(),
    renderDarkCase(),
  ];
  const custom = currentCustomPhrase();
  if (custom) {
    cards.push(renderCustomCase(custom));
  }
  evidenceWall.innerHTML = cards.join("");
}

function renderReportProfileGrid() {
  reportProfileGrid.innerHTML = Object.entries(REPORT_PROFILES)
    .map(
      ([id, profile]) => `
        <button class="profile-card ${state.selectedReportProfile === id ? "is-selected" : ""}" data-report-profile="${id}">
          <strong>${escapeHtml(profile.title)}</strong>
          <span>${escapeHtml(profile.detail)}</span>
        </button>
      `
    )
    .join("");

  reportProfileGrid.querySelectorAll("[data-report-profile]").forEach((button) => {
    button.addEventListener("click", () => {
      state.selectedReportProfile = button.dataset.reportProfile;
      clearReport();
      renderSummary();
      renderReportProfileGrid();
      runStatus.textContent = `Report setting: ${REPORT_PROFILES[state.selectedReportProfile].title}.`;
    });
  });
}

function renderRegistry() {
  if (!state.registry.length) {
    registryList.innerHTML = `<div class="empty-state">No saved fonts yet.</div>`;
    return;
  }

  registryList.innerHTML = state.registry
    .map((asset) => {
      const selectedAssetId = state.selectedSource.asset ? state.selectedSource.asset.id : "";
      const isSelected = selectedAssetId === asset.id;
      const title = escapeHtml(asset.family_name || asset.file_name);
      return `
        <article class="registry-card ${isSelected ? "is-selected" : ""}">
          <div>
            <div class="registry-title">${title}</div>
            <p class="registry-meta">${escapeHtml(asset.file_name)}</p>
            <p class="registry-meta">${escapeHtml(asset.license_normalized)} · ${escapeHtml(titleCaseStatus(asset.status))}</p>
          </div>
          <div class="registry-actions">
            <span class="status-pill ${asset.status}">${titleCaseStatus(asset.status)}</span>
            <button class="secondary-button" data-use-asset="${asset.id}">Use this font</button>
          </div>
        </article>
      `;
    })
    .join("");

  registryList.querySelectorAll("[data-use-asset]").forEach((button) => {
    button.addEventListener("click", async () => {
      const asset = state.registry.find((entry) => entry.id === button.dataset.useAsset);
      if (!asset) {
        return;
      }

      const starter = STARTER_FONTS.find((entry) => entry.uploadFileName === asset.file_name);
      let previewFont = "system-ui, sans-serif";

      if (starter) {
        state.selectedStarterId = starter.id;
        await ensureStarterFontLoaded(starter);
        previewFont = `"${starter.family}", ${starter.fallback}`;
      } else {
        runStatus.textContent = `Loading ${asset.family_name || asset.file_name} from saved fonts...`;
        try {
          previewFont = await ensureRegistryAssetLoaded(asset);
        } catch (error) {
          runStatus.textContent = `Could not preview ${asset.family_name || asset.file_name}: ${error.message}`;
        }
      }

      setSelectedSource(
        {
          type: "registry",
          label: asset.family_name || asset.file_name,
          detail: "Saved font",
          asset,
          previewFont,
        },
        `Showing ${asset.family_name || asset.file_name} across the page.`
      );

      renderAll();
    });
  });
}

function renderReport(report) {
  const scorePercent = Math.round(report.measurements.score * 100);
  const coveragePercent = Math.round(report.measurements.estimated_coverage * 100);
  metricGrid.innerHTML = `
    <article class="metric-card">
      <span>Score</span>
      <strong>${scorePercent}</strong>
    </article>
    <article class="metric-card">
      <span>Coverage</span>
      <strong>${coveragePercent}%</strong>
    </article>
    <article class="metric-card">
      <span>Density</span>
      <strong>${report.measurements.line_density.toFixed(1)}</strong>
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
  renderSummary();
  renderStarterGrid();
  renderEvidenceWall();
  renderReportProfileGrid();
  renderRegistry();

  if (state.lastReport) {
    renderReport(state.lastReport);
  }
}

async function refreshRegistry() {
  registryList.innerHTML = `<div class="empty-state">Refreshing saved fonts...</div>`;
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
    registryList.innerHTML = `<div class="empty-state">Could not load saved fonts: ${error.message}</div>`;
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

  if (state.selectedSource.type === "url" && state.selectedSource.asset) {
    state.selectedSource.asset =
      state.registry.find((asset) => asset.id === state.selectedSource.asset.id) || state.selectedSource.asset;
  }

  renderRegistry();
}

async function uploadPendingFile() {
  if (!state.pendingUpload) {
    throw new Error("choose a local font file first");
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

  uploadMsg.textContent = `Saved ${state.pendingUpload.name}. There are now ${payload.total} saved font files.`;
  await refreshRegistry();
  const asset = findRegistryMatchByFileName(state.pendingUpload.name);
  if (!asset) {
    throw new Error("upload finished but the saved font was not found");
  }

  setSelectedSource(
    {
      type: "upload",
      label: asset.family_name || state.pendingUpload.name,
      detail: "Uploaded from this page",
      previewFont: state.selectedSource.previewFont,
      asset,
    },
    `Saved ${asset.family_name || state.pendingUpload.name} and kept it on the page.`
  );

  renderAll();
  return asset;
}

async function loadFontFromUrl() {
  const url = fontUrlInput.value.trim();
  if (!url) {
    urlMsg.textContent = "Paste a direct font file URL first.";
    return;
  }

  loadUrlBtn.disabled = true;
  try {
    urlMsg.textContent = "Loading the font file URL...";
    runStatus.textContent = "Fetching the font file URL...";

    const response = await fetch("/api/fonts/ingest-url", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        url,
        declared_license: licenseSelect.value,
      }),
    });
    const payload = await response.json();
    if (!response.ok) {
      throw new Error(payload.error || "could not load the font URL");
    }

    await refreshRegistry();
    const asset = findRegistryMatchByFileName(payload.file_name);
    if (!asset) {
      throw new Error("the font URL loaded, but the saved font was not found");
    }

    const previewFont = await ensureRegistryAssetLoaded(asset);
    setSelectedSource(
      {
        type: "url",
        label: asset.family_name || asset.file_name,
        detail: "Loaded from a font file URL",
        asset,
        previewFont,
      },
      `Showing ${asset.family_name || asset.file_name} across the page.`
    );
    urlMsg.textContent = `Loaded ${asset.family_name || asset.file_name} from the web.`;
    renderAll();
  } catch (error) {
    urlMsg.textContent = `Could not load that URL: ${error.message}`;
    runStatus.textContent = `Could not load that URL: ${error.message}`;
  } finally {
    loadUrlBtn.disabled = false;
  }
}

async function ensureBenchAsset() {
  if (state.selectedSource.asset) {
    return state.selectedSource.asset;
  }

  if (state.selectedSource.type === "starter") {
    const starter = findStarter(state.selectedSource.starterId);
    runStatus.textContent = `Saving ${starter.label} so the report can run...`;
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
      throw new Error(payload.error || "example font save failed");
    }

    await refreshRegistry();
    const asset = findRegistryMatchByFileName(starter.uploadFileName);
    if (!asset) {
      throw new Error("the example font saved, but it was not found");
    }
    state.selectedSource.asset = asset;
    return asset;
  }

  if (state.selectedSource.type === "upload") {
    return uploadPendingFile();
  }

  throw new Error("load a font first");
}

async function runReport() {
  generateReportBtn.disabled = true;
  try {
    const asset = await ensureBenchAsset();
    runStatus.textContent =
      `Generating the ${REPORT_PROFILES[state.selectedReportProfile].title} report for ${asset.family_name || asset.file_name}...`;
    const response = await fetch(`/api/fonts/${asset.id}/report?profile=${state.selectedReportProfile}`);
    const payload = await response.json();
    if (!response.ok) {
      throw new Error(payload.error || "report failed");
    }
    state.lastReport = payload;
    state.lastReportProfile = state.selectedReportProfile;
    renderSummary();
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
  uploadMsg.textContent = "Showing the uploaded file across the page.";

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
      detail: "Local preview from this page",
      previewFont: `"${family}", system-ui, sans-serif`,
      asset: findRegistryMatchByFileName(file.name),
    },
    `Showing ${file.name} across the page. Save it when you want a report.`
  );
  renderAll();
}

async function initialize() {
  await ensureStarterFontLoaded(STARTER_FONTS[0]);
  applyPageFont(state.selectedSource.previewFont);
  renderAll();
  await refreshRegistry();

  const initialUrl = new URL(window.location.href).searchParams.get("fontUrl");
  if (initialUrl) {
    fontUrlInput.value = initialUrl;
    await loadFontFromUrl();
    return;
  }

  runStatus.textContent = `Showing ${state.selectedSource.label} across the page.`;
}

customPhraseInput.addEventListener("input", () => {
  runStatus.textContent = currentCustomPhrase()
    ? "Your custom line is now part of the reading checks."
    : `Showing ${state.selectedSource.label} across the page.`;
  renderEvidenceWall();
});

fileInput.addEventListener("change", () => {
  const [file] = fileInput.files;
  if (!file) {
    return;
  }
  updateUploadPreview(file);
});

loadUrlBtn.addEventListener("click", loadFontFromUrl);

fontUrlInput.addEventListener("keydown", (event) => {
  if (event.key === "Enter") {
    event.preventDefault();
    loadFontFromUrl();
  }
});

uploadBtn.addEventListener("click", async () => {
  uploadBtn.disabled = true;
  try {
    await uploadPendingFile();
  } catch (error) {
    uploadMsg.textContent = `Could not save the upload: ${error.message}`;
  } finally {
    uploadBtn.disabled = !state.pendingUpload;
  }
});

generateReportBtn.addEventListener("click", runReport);
refreshRegistryBtn.addEventListener("click", refreshRegistry);

initialize().catch((error) => {
  runStatus.textContent = `Startup failed: ${error.message}`;
});
