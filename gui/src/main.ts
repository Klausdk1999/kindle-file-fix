export {};

declare global {
    interface Window {
        __TAURI__: {
            core: {
                invoke: <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
            };
        };
    }
}

interface FixReport {
    filename: string;
    format: string;
    fixes: string[];
    warnings: string[];
    has_fixes: boolean;
    error: string | null;
}

const dropzone = document.getElementById("dropzone")!;
const selectBtn = document.getElementById("selectBtn")!;
const resultsEl = document.getElementById("results")!;
const statusEl = document.getElementById("status")! as HTMLDivElement;
const keepName = document.getElementById("keepName") as HTMLInputElement;

function showStatus(message: string) {
    statusEl.textContent = message;
    statusEl.style.display = "block";
}

function hideStatus() {
    statusEl.style.display = "none";
}

selectBtn.addEventListener("click", async () => {
    try {
        // Use Tauri's file dialog
        const { invoke } = window.__TAURI__.core;
        const selected = await invoke<string[] | null>("plugin:dialog|open", {
            multiple: true,
            filters: [{ name: "Ebooks", extensions: ["epub", "mobi", "azw3"] }],
        });

        if (selected && selected.length > 0) {
            await processFiles(selected);
        }
    } catch (err) {
        console.error("File dialog error:", err);
    }
});

dropzone.addEventListener("dragover", (e) => {
    e.preventDefault();
    dropzone.classList.add("dragover");
});

dropzone.addEventListener("dragleave", () => {
    dropzone.classList.remove("dragover");
});

dropzone.addEventListener("drop", async (e) => {
    e.preventDefault();
    dropzone.classList.remove("dragover");

    const files = e.dataTransfer?.files;
    if (files && files.length > 0) {
        const paths: string[] = [];
        for (let i = 0; i < files.length; i++) {
            // In Tauri, dropped files have a path property
            const file = files[i] as File & { path?: string };
            if (file.path) {
                paths.push(file.path);
            }
        }
        if (paths.length > 0) {
            await processFiles(paths);
        }
    }
});

async function processFiles(paths: string[]) {
    const { invoke } = window.__TAURI__.core;

    showStatus(`Processing ${paths.length} file(s)...`);
    resultsEl.innerHTML = "";

    try {
        const reports = await invoke<FixReport[]>("process_files", {
            paths,
            language: null,
            keepName: keepName.checked,
        });

        hideStatus();
        renderResults(reports);
    } catch (err) {
        hideStatus();
        resultsEl.innerHTML = `<div class="result-item"><p class="error">Error: ${err}</p></div>`;
    }
}

function renderResults(reports: FixReport[]) {
    resultsEl.innerHTML = "";

    for (const report of reports) {
        const div = document.createElement("div");
        div.className = "result-item";

        let statusHtml = "";
        if (report.error) {
            statusHtml = `<p class="error">${escapeHtml(report.error)}</p>`;
        } else if (report.fixes.length > 0) {
            statusHtml = `<ul>${report.fixes.map((f) => `<li class="fix">${escapeHtml(f)}</li>`).join("")}</ul>`;
        } else {
            statusHtml = `<p class="ok">No issues found. File repacked successfully.</p>`;
        }

        if (report.warnings.length > 0) {
            statusHtml += `<ul>${report.warnings.map((w) => `<li class="warning">${escapeHtml(w)}</li>`).join("")}</ul>`;
        }

        div.innerHTML = `<h3>${escapeHtml(report.filename)} <small>(${escapeHtml(report.format)})</small></h3>${statusHtml}`;
        resultsEl.appendChild(div);
    }
}

function escapeHtml(text: string): string {
    const div = document.createElement("div");
    div.textContent = text;
    return div.innerHTML;
}
