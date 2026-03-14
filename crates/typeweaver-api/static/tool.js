const STARTER_FONTS = [
  {
    id: "roboto",
    label: "Roboto",
    family: "Roboto",
    familyQuery: "Roboto:wght@400;700",
    fallback: "system-ui, sans-serif",
    uploadFileName: "Roboto-Regular.ttf",
    licenseText: "Apache License Version 2.0",
  },
  {
    id: "roboto_condensed",
    label: "Roboto Condensed",
    family: "Roboto Condensed",
    familyQuery: "Roboto+Condensed:wght@400;700",
    fallback: "\"Arial Narrow\", sans-serif",
    uploadFileName: "RobotoCondensed-Regular.ttf",
    licenseText: "Apache License Version 2.0",
  },
  {
    id: "roboto_slab",
    label: "Roboto Slab",
    family: "Roboto Slab",
    familyQuery: "Roboto+Slab:wght@400;700",
    fallback: "Georgia, serif",
    uploadFileName: "RobotoSlab-Regular.ttf",
    licenseText: "Apache License Version 2.0",
  },
];

const SCENARIOS = {
  web_light_default: {
    title: "Web light default",
    short: "bright · roomy",
  },
  mobile_dark_low_contrast: {
    title: "Mobile dark low contrast",
    short: "dim · tight",
  },
};

const SCORE_NARRATIVES = [
  { limit: 0.85, text: "Strong result. The font holds the selected profile." },
  { limit: 0.65, text: "Good overall. Pressure is visible but controlled." },
  { limit: 0.4, text: "Mixed result. Expect visible failure points." },
  { limit: 0, text: "Fragile result. The profile exposes real readability problems." },
];

const BENCH_CORPUS =
  "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()-_=+[]{};:'\\\",.<>/?\\\\|`~ O/0 I/l/1 S/5 B/8 rn/m cl/d";

const starterGrid = document.getElementById("starterGrid");
const currentFontLabel = document.getElementById("currentFontLabel");
const heroTitle = document.getElementById("heroTitle");
const labSummary = document.getElementById("labSummary");
const evidenceWall = document.getElementById("evidenceWall");
const customPhraseInput = document.getElementById("customPhraseInput");
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

function fontStyle(extra = "") {
  const previewFont = state.selectedSource.previewFont || "system-ui, sans-serif";
  return `font-family:${previewFont};${extra}`;
}

function currentCustomPhrase() {
  return customPhraseInput.value.trim();
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

function renderStarterGrid() {
  starterGrid.innerHTML = STARTER_FONTS.map((starter) => {
    const selected =
      (state.selectedSource.type === "starter" && state.selectedSource.starterId === starter.id) ||
      (state.selectedSource.asset && state.selectedSource.asset.file_name === starter.uploadFileName);
    return `
      <button class="starter-card ${selected ? "is-selected" : ""}" data-starter="${starter.id}">
        <div class="starter-swatch" style="font-family:'${starter.family}', ${starter.fallback};">Aa 01 rn</div>
        <strong>${starter.label}</strong>
        <span>${starter.family}</span>
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
        `Showing ${starter.label} across the stress wall.`
      );
      renderAll();
    });
  });
}

function renderHeroMeta() {
  currentFontLabel.textContent = state.selectedSource.label;
  heroTitle.textContent = `${state.selectedSource.label} under pressure.`;
  const custom = currentCustomPhrase();
  labSummary.innerHTML = `
    <div class="summary-card">
      <strong>${escapeHtml(state.selectedSource.label)}</strong>
      <span>font</span>
    </div>
    <div class="summary-card">
      <strong>${escapeHtml(state.selectedSource.detail)}</strong>
      <span>source</span>
    </div>
    <div class="summary-card">
      <strong>6 automatic</strong>
      <span>stress scenes</span>
    </div>
    <div class="summary-card">
      <strong>${custom ? "4 built-in + 1 custom" : "4 built-in"}</strong>
      <span>specimen phrases</span>
    </div>
  `;
}

function renderLabCard({
  title,
  detail,
  beforeLabel,
  afterLabel,
  beforeClass,
  afterClass,
  beforeContent,
  afterContent,
}) {
  return `
    <article class="lab-card">
      <div class="lab-card-head">
        <div>
          <span class="lab-kicker">${escapeHtml(title)}</span>
          <strong>${escapeHtml(detail)}</strong>
        </div>
      </div>
      <div class="comparison-grid">
        <section class="scene-card ${beforeClass}">
          <span class="scene-label">${escapeHtml(beforeLabel)}</span>
          ${beforeContent}
        </section>
        <section class="scene-card ${afterClass}">
          <span class="scene-label">${escapeHtml(afterLabel)}</span>
          ${afterContent}
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
          <div class="mini-ui-row">
            <span>${escapeHtml(left)}</span>
            <strong>${escapeHtml(right)}</strong>
          </div>
        `
      )
      .join("");

  return renderLabCard({
    title: "Tiny UI",
    detail: "numbers · dates · passcodes",
    beforeLabel: "clear",
    afterLabel: "11px",
    beforeClass: "is-paper",
    afterClass: "is-sand",
    beforeContent: `<div class="mini-ui" style="${fontStyle()}">${buildRows()}</div>`,
    afterContent: `<div class="mini-ui is-stress" style="${fontStyle()}">${buildRows()}</div>`,
  });
}

function renderContrastCase() {
  const text = "Order 1001 ships to 10 I/O labs.";
  const subtle = "1001 · 05/18 · O0 · I l 1";
  return renderLabCard({
    title: "Low contrast",
    detail: "edges fade first",
    beforeLabel: "clear",
    afterLabel: "washed",
    beforeClass: "is-paper",
    afterClass: "is-sand",
    beforeContent: `
      <div class="phrase-stack" style="${fontStyle()}">
        <p class="phrase-sample">${escapeHtml(text)}</p>
        <span class="scene-subtle">${escapeHtml(subtle)}</span>
      </div>
    `,
    afterContent: `
      <div class="phrase-stack" style="${fontStyle()}">
        <p class="phrase-sample is-fade">${escapeHtml(text)}</p>
        <span class="scene-subtle">${escapeHtml(subtle)}</span>
      </div>
    `,
  });
}

function renderBlurCase() {
  const text = "Returns, refunds, receipts.";
  const subtle = "rn / m · cl / d · 8B / S5";
  return renderLabCard({
    title: "Blur",
    detail: "soft rendering",
    beforeLabel: "sharp",
    afterLabel: "blurred",
    beforeClass: "is-paper",
    afterClass: "is-sand",
    beforeContent: `
      <div class="phrase-stack" style="${fontStyle()}">
        <p class="phrase-sample">${escapeHtml(text)}</p>
        <span class="scene-subtle">${escapeHtml(subtle)}</span>
      </div>
    `,
    afterContent: `
      <div class="phrase-stack" style="${fontStyle()}">
        <p class="phrase-sample is-blur">${escapeHtml(text)}</p>
        <span class="scene-subtle">${escapeHtml(subtle)}</span>
      </div>
    `,
  });
}

function renderConfusableCase() {
  const pairs = ["I l 1", "O 0", "rn m", "cl d", "S 5", "B 8"];
  return renderLabCard({
    title: "Confusables",
    detail: "I l 1 · O 0 · rn m",
    beforeLabel: "spaced",
    afterLabel: "squeezed",
    beforeClass: "is-paper",
    afterClass: "is-sand",
    beforeContent: `
      <div class="pair-grid" style="${fontStyle()}">
        ${pairs.map((pair) => `<div class="pair-cell">${escapeHtml(pair)}</div>`).join("")}
      </div>
    `,
    afterContent: `
      <div class="phrase-stack" style="${fontStyle()}">
        <p class="pair-line is-squeezed" style="${fontStyle("filter:blur(0.45px); opacity:0.78; transform:scaleX(0.96); transform-origin:left center;")}">Il1 O0 rnm cld S5 B8</p>
        <span class="scene-subtle">tight + dim</span>
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
    ["Refunds", "I l 1"],
    ["Gate C12", "10:31 PM"],
    ["rn / m", "cl / d"],
  ];
  return renderLabCard({
    title: "Dense UI",
    detail: "crowded interface",
    beforeLabel: "roomy",
    afterLabel: "packed",
    beforeClass: "is-paper",
    afterClass: "is-sand",
    beforeContent: `
      <div class="dense-board" style="${fontStyle()}">
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
    afterContent: `
      <div class="dense-board is-stress" style="${fontStyle()}">
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

  return renderLabCard({
    title: "Dark mode",
    detail: "dim screen · smaller type",
    beforeLabel: "light",
    afterLabel: "dim dark",
    beforeClass: "is-paper",
    afterClass: "is-night",
    beforeContent: `<div class="message-stack" style="${fontStyle()}">${buildMessages()}</div>`,
    afterContent: `<div class="message-stack is-dark" style="${fontStyle("font-size:0.92em;")}">${buildMessages()}</div>`,
  });
}

function renderCustomCase(text) {
  return renderLabCard({
    title: "Custom line",
    detail: "your phrase",
    beforeLabel: "clear",
    afterLabel: "stress",
    beforeClass: "is-paper",
    afterClass: "is-night",
    beforeContent: `
      <div class="phrase-stack" style="${fontStyle()}">
        <p class="phrase-sample">${escapeHtml(text)}</p>
        <span class="scene-subtle">custom</span>
      </div>
    `,
    afterContent: `
      <div class="phrase-stack" style="${fontStyle()}">
        <p class="phrase-sample is-fade is-blur is-tight">${escapeHtml(text)}</p>
        <span class="scene-subtle">tiny + dim + blur</span>
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
  reportProfileGrid.innerHTML = Object.entries(SCENARIOS)
    .map(
      ([id, scenario]) => `
        <button class="profile-card ${state.selectedReportProfile === id ? "is-selected" : ""}" data-report-profile="${id}">
          <strong>${escapeHtml(id)}</strong>
          <span>${escapeHtml(scenario.short)}</span>
        </button>
      `
    )
    .join("");

  reportProfileGrid.querySelectorAll("[data-report-profile]").forEach((button) => {
    button.addEventListener("click", () => {
      state.selectedReportProfile = button.dataset.reportProfile;
      clearReport();
      runStatus.textContent = `Report profile set to ${state.selectedReportProfile}.`;
      renderAll();
    });
  });
}

function renderRegistry() {
  if (!state.registry.length) {
    registryList.innerHTML = `<div class="empty-state">Registry is empty.</div>`;
    return;
  }

  registryList.innerHTML = state.registry
    .map((asset) => {
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
        runStatus.textContent = `Loading ${asset.family_name || asset.file_name} from the registry...`;
        try {
          previewFont = await ensureRegistryAssetLoaded(asset);
        } catch (error) {
          runStatus.textContent = `Preview fallback for ${asset.family_name || asset.file_name}: ${error.message}`;
        }
      }

      setSelectedSource(
        {
          type: "registry",
          label: asset.family_name || asset.file_name,
          detail: "Existing registry asset",
          asset,
          previewFont,
        },
        `Showing ${asset.family_name || asset.file_name} from the registry.`
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
      <strong>${scorePercent}</strong>
      <span>score</span>
    </article>
    <article class="metric-card">
      <strong>${coveragePercent}</strong>
      <span>coverage</span>
    </article>
    <article class="metric-card">
      <strong>${report.measurements.line_density.toFixed(1)}</strong>
      <span>density</span>
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
  renderHeroMeta();
  renderEvidenceWall();
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
    `Uploaded ${state.pendingUpload.name} and switched the wall to it.`
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
    runStatus.textContent =
      `Generating ${SCENARIOS[state.selectedReportProfile].title} report for ${asset.family_name || asset.file_name}...`;
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
  uploadMsg.textContent = "Previewing the local file on the wall.";

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
    `Previewing ${file.name} across the stress wall.`
  );
  renderAll();
}

async function initialize() {
  await ensureStarterFontLoaded(STARTER_FONTS[0]);
  renderAll();
  await refreshRegistry();
  runStatus.textContent = `Showing ${state.selectedSource.label} across 6 stress scenes.`;
}

customPhraseInput.addEventListener("input", () => {
  runStatus.textContent = currentCustomPhrase()
    ? "Custom line added to the wall."
    : `Showing ${state.selectedSource.label} across 6 stress scenes.`;
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
