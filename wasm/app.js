// Butabuti WASM Interface - Embroidery File Converter
// Compiled from TypeScript for type safety and error-free operation

import init, {
    convert_pattern,
    get_pattern_info,
    export_to_svg,
    list_formats,
} from "./pkg/butabuti.js";

// Global state
let wasmReady = false;
/** @type {Uint8Array | null} */
let fileData = null;
/** @type {string | null} */
let fileFormat = null;
let fileName = "";

/**
 * Initialize WASM module and set up the application
 */
async function initWasm() {
    try {
        console.log("Initializing WASM module...");
        await init();
        wasmReady = true;
        console.log("WASM module initialized successfully");

        const formatsJson = list_formats();
        console.log("Formats JSON:", formatsJson);

        const formats = JSON.parse(formatsJson);
        console.log("Parsed formats:", formats);

        populateFormatSelect(formats);
        updateFileAcceptTypes(formats);
        setupDragAndDrop();
        setupEventListeners();

        showSuccess("Ready! You can now upload a file.");
        console.log("Application ready");
    } catch (err) {
        const errorMsg =
            "Failed to load WASM module. Make sure you're serving this page via HTTP server (not file://). Run: npx http-server -p 8000";
        showError(errorMsg);
        console.error("WASM initialization error:", err);
    }
}

/**
 * Populate the format selector with available formats
 * @param {any} formats - Format data from WASM
 */
function populateFormatSelect(formats) {
    const select = /** @type {HTMLSelectElement | null} */ (
        document.getElementById("outputFormat")
    );
    if (!select) {
        console.error("Output format select not found");
        return;
    }

    select.innerHTML = '<option value="">Select format...</option>';

    const embroideryFormats = formats.output_formats.filter(
        /** @param {any} f */ (f) =>
            !["svg", "txt", "csv", "json"].includes(f.name)
    );
    const exportFormats = formats.output_formats.filter(
        /** @param {any} f */ (f) =>
            ["svg", "txt", "csv", "json"].includes(f.name)
    );

    if (embroideryFormats.length > 0) {
        const embGroup = document.createElement("optgroup");
        embGroup.label = "Embroidery Formats";
        embroideryFormats.forEach(
            /** @param {any} fmt */ (fmt) => {
                const option = document.createElement("option");
                option.value = fmt.name;
                option.textContent = fmt.display_name;
                embGroup.appendChild(option);
            }
        );
        select.appendChild(embGroup);
    }

    if (exportFormats.length > 0) {
        const expGroup = document.createElement("optgroup");
        expGroup.label = "Export Formats";
        exportFormats.forEach(
            /** @param {any} fmt */ (fmt) => {
                const option = document.createElement("option");
                option.value = fmt.name;
                option.textContent = fmt.display_name;
                if (fmt.name === "svg") option.selected = true;
                expGroup.appendChild(option);
            }
        );
        select.appendChild(expGroup);
    }

    console.log(
        `Populated ${embroideryFormats.length + exportFormats.length} formats`
    );
}

/**
 * Update file input to accept valid embroidery file extensions
 * @param {any} formats - Format data from WASM
 */
function updateFileAcceptTypes(formats) {
    const fileInput = /** @type {HTMLInputElement | null} */ (
        document.getElementById("fileInput")
    );
    if (!fileInput) {
        console.error("File input not found");
        return;
    }

    const extensions = formats.input_formats.flatMap(
        /** @param {any} f */ (f) => f.extensions
    );
    fileInput.accept = extensions
        .map(/** @param {any} ext */ (ext) => "." + ext)
        .join(",");
    console.log("Accepting file types:", fileInput.accept);
}

/**
 * Handle file selection (from drag-drop or file input)
 * @param {File} file - The file to handle
 */
function handleFileSelection(file) {
    if (!file) {
        console.warn("No file selected");
        return;
    }

    console.log("File selected:", file.name, file.size, "bytes");
    fileName = file.name;
    const ext = file.name.split(".").pop()?.toLowerCase() || "";
    fileFormat = ext;

    const reader = new FileReader();
    reader.onload = (e) => {
        if (!e.target?.result || typeof e.target.result === "string") {
            showError("Failed to read file as binary data");
            return;
        }

        fileData = new Uint8Array(e.target.result);
        console.log("File loaded:", fileData.length, "bytes");

        const convertBtn = /** @type {HTMLButtonElement | null} */ (
            document.getElementById("convertBtn")
        );
        const infoBtn = /** @type {HTMLButtonElement | null} */ (
            document.getElementById("infoBtn")
        );
        const previewBtn = /** @type {HTMLButtonElement | null} */ (
            document.getElementById("previewBtn")
        );

        if (convertBtn) convertBtn.disabled = false;
        if (infoBtn) infoBtn.disabled = false;
        if (previewBtn) previewBtn.disabled = false;

        const fileInfo = document.getElementById("fileInfo");
        if (fileInfo) {
            fileInfo.className = "file-info show";
            fileInfo.innerHTML = `File: ${file.name} • Size: ${formatBytes(
                fileData.length
            )} • Format: ${ext.toUpperCase()}`;
        }

        showSuccess("File loaded successfully");
    };
    reader.onerror = () => {
        console.error("File read error:", reader.error);
        showError("Failed to read file");
    };
    reader.readAsArrayBuffer(file);
}

/**
 * Set up drag and drop functionality
 */
function setupDragAndDrop() {
    const dropZone = document.getElementById("dropZone");
    const fileInput = document.getElementById("fileInput");

    if (!dropZone || !fileInput) {
        console.error("Drop zone or file input not found");
        return;
    }

    dropZone.addEventListener("click", () => {
        console.log("Drop zone clicked");
        fileInput.click();
    });

    ["dragenter", "dragover", "dragleave", "drop"].forEach((eventName) => {
        dropZone.addEventListener(
            eventName,
            (e) => {
                e.preventDefault();
                e.stopPropagation();
            },
            false
        );
    });

    ["dragenter", "dragover"].forEach((eventName) => {
        dropZone.addEventListener(
            eventName,
            () => {
                dropZone.classList.add("drag-over");
            },
            false
        );
    });

    ["dragleave", "drop"].forEach((eventName) => {
        dropZone.addEventListener(
            eventName,
            () => {
                dropZone.classList.remove("drag-over");
            },
            false
        );
    });

    dropZone.addEventListener(
        "drop",
        (e) => {
            if (e.dataTransfer?.files && e.dataTransfer.files.length > 0) {
                console.log("Files dropped:", e.dataTransfer.files.length);
                handleFileSelection(e.dataTransfer.files[0]);
            }
        },
        false
    );

    fileInput.addEventListener("change", (e) => {
        const target = /** @type {HTMLInputElement} */ (e.target);
        if (target?.files && target.files.length > 0) {
            handleFileSelection(target.files[0]);
        }
    });

    console.log("Drag and drop initialized");
}

/**
 * Set up button event listeners
 */
function setupEventListeners() {
    const convertBtn = document.getElementById("convertBtn");
    const infoBtn = document.getElementById("infoBtn");
    const previewBtn = document.getElementById("previewBtn");

    if (convertBtn) {
        convertBtn.addEventListener("click", handleConvert);
    }

    if (infoBtn) {
        infoBtn.addEventListener("click", handleInfo);
    }

    if (previewBtn) {
        previewBtn.addEventListener("click", handlePreview);
    }

    console.log("Event listeners initialized");
}

/**
 * Handle convert button click
 */
async function handleConvert() {
    if (!wasmReady || !fileData || !fileFormat) {
        console.warn("WASM not ready or no file data");
        return;
    }

    const outputFormatSelect = /** @type {HTMLSelectElement | null} */ (
        document.getElementById("outputFormat")
    );
    const outputFormat = outputFormatSelect?.value || "";

    if (!outputFormat) {
        showError("Please select an output format");
        return;
    }

    console.log(`Converting to ${outputFormat}...`);

    try {
        const result = convert_pattern(fileData, fileFormat, outputFormat);
        console.log("Conversion successful, result size:", result.length);

        // Create blob from Uint8Array (cast to any to avoid type issues)
        const blob = new Blob([/** @type {any} */ (result)], {
            type: "application/octet-stream",
        });
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        const baseName = fileName.substring(0, fileName.lastIndexOf("."));
        a.download = `${baseName}.${outputFormat}`;
        a.click();
        URL.revokeObjectURL(url);

        showSuccess(`Successfully converted to ${outputFormat.toUpperCase()}`);
    } catch (err) {
        console.error("Conversion error:", err);
        const errorMsg = err instanceof Error ? err.message : String(err);
        showError("Conversion failed: " + errorMsg);
    }
}

/**
 * Handle info button click
 */
async function handleInfo() {
    if (!wasmReady || !fileData || !fileFormat) {
        console.warn("WASM not ready or no file data");
        return;
    }

    console.log("Getting pattern info...");

    try {
        const infoJson = get_pattern_info(fileData, fileFormat);
        console.log("Info JSON:", infoJson);

        const info = JSON.parse(infoJson);
        console.log("Pattern info:", info);

        let html = '<div class="output-title">Pattern Information</div>';
        html += '<div class="stats-grid">';
        html += `<div class="stat-item"><div class="stat-label">Stitches</div><div class="stat-value">${info.stitch_count.toLocaleString()}</div></div>`;
        html += `<div class="stat-item"><div class="stat-label">Colors</div><div class="stat-value">${info.color_count}</div></div>`;
        html += `<div class="stat-item"><div class="stat-label">Width</div><div class="stat-value">${info.width_mm.toFixed(
            1
        )}mm</div></div>`;
        html += `<div class="stat-item"><div class="stat-label">Height</div><div class="stat-value">${info.height_mm.toFixed(
            1
        )}mm</div></div>`;
        html += "</div>";

        if (info.colors && info.colors.length > 0) {
            html += '<div class="section-title">Colors</div>';
            html += '<div class="color-list">';
            info.colors.forEach(
                /** @param {any} color @param {number} idx */ (color, idx) => {
                    const rgb = `rgb(${color.red}, ${color.green}, ${color.blue})`;
                    html += `<div class="color-chip">`;
                    html += `<div class="color-swatch" style="background-color: ${rgb}"></div>`;
                    html += `<span>${idx + 1}. ${color.description}</span>`;
                    html += `</div>`;
                }
            );
            html += "</div>";
        }

        showOutput(html);
    } catch (err) {
        console.error("Info error:", err);
        const errorMsg = err instanceof Error ? err.message : String(err);
        showError("Failed to get pattern info: " + errorMsg);
    }
}

/**
 * Handle preview button click
 */
async function handlePreview() {
    if (!wasmReady || !fileData || !fileFormat) {
        console.warn("WASM not ready or no file data");
        return;
    }

    const qualitySelect = /** @type {HTMLSelectElement | null} */ (
        document.getElementById("svgQuality")
    );
    const quality = qualitySelect?.value || "high";

    console.log(`Generating SVG preview (quality: ${quality})...`);

    try {
        // Note: Using export_to_svg (quality parameter not yet available in WASM build)
        const svg = export_to_svg(fileData, fileFormat);
        console.log("SVG generated, length:", svg.length);

        let html = '<div class="output-title">SVG Preview</div>';
        html += `<div id="svgPreview">${svg}</div>`;

        showOutput(html);
    } catch (err) {
        console.error("Preview error:", err);
        const errorMsg = err instanceof Error ? err.message : String(err);
        showError("Failed to generate preview: " + errorMsg);
    }
}

/**
 * Format bytes to human-readable string
 * @param {number} bytes - Number of bytes
 * @returns {string} Formatted string
 */
function formatBytes(bytes) {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + " " + sizes[i];
}

/**
 * Show output in the output section
 * @param {string} html - HTML content to display
 */
function showOutput(html) {
    const output = document.getElementById("output");
    if (!output) {
        console.error("Output element not found");
        return;
    }
    output.innerHTML = html;
    output.className = "show";
    hideMessages();
}

/**
 * Show success message
 * @param {string} message - Success message to display
 */
function showSuccess(message) {
    const success = document.getElementById("successMessage");
    const error = document.getElementById("errorMessage");

    if (!success) {
        console.error("Success message element not found");
        return;
    }

    success.textContent = message;
    success.className = "success show";

    if (error) {
        error.className = "error";
    }

    setTimeout(() => {
        success.className = "success";
    }, 3000);

    console.log("Success:", message);
}

/**
 * Show error message
 * @param {string} message - Error message to display
 */
function showError(message) {
    const error = document.getElementById("errorMessage");
    const success = document.getElementById("successMessage");

    if (!error) {
        console.error("Error message element not found");
        return;
    }

    error.textContent = message;
    error.className = "error show";

    if (success) {
        success.className = "success";
    }

    console.error("Error:", message);
}

/**
 * Hide all messages
 */
function hideMessages() {
    const error = document.getElementById("errorMessage");
    const success = document.getElementById("successMessage");

    if (error) error.className = "error";
    if (success) success.className = "success";
}

// Initialize when DOM is ready
if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initWasm);
} else {
    initWasm();
}
