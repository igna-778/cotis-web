// renderer.js — HTML DOM renderer
let lastTime = performance.now();
let now = performance.now();

// Returns a delta time in milliseconds (time between frames)
export function get_delta_time_ms() {
    const delta = (now - lastTime);
    return Math.max(delta, 1);
}

export async function waitForNextFrame() {
    await new Promise((resolve) => {
        requestAnimationFrame(() => resolve());
    });
}

// Returns the window dimensions as an array [width, height]
export function window_dimensions() {
    return [window.innerWidth, window.innerHeight];
}

// Draw a frame given an array of commands
export function draw_frame(commands) {
    ensureRenderRoot();
    lastTime = now;
    now = performance.now();
    renderLoopHTML(commands);
}

// Reusable hidden element for measuring text with the same font/size as the HTML text renderer
let textMeasureEl = null;

const DEFAULT_LINE_HEIGHT = 1.2;

function getTextMeasureElement() {
    if (textMeasureEl) return textMeasureEl;
    textMeasureEl = document.createElement("div");
    textMeasureEl.setAttribute("aria-hidden", "true");
    textMeasureEl.style.cssText = "position:absolute;left:-9999px;top:0;visibility:hidden;display:inline-block;box-sizing:border-box;margin:0;padding:0;border:0;";
    document.body.appendChild(textMeasureEl);
    return textMeasureEl;
}

// Measure text and return an array [width, height]. Supports multi-line (\\n): returns [max line width, total height].
// Keep whitespace handling aligned with runtime text rendering so trailing/duplicate spaces
// (including a lone " ") occupy the same measured width they render with.
export function text_measuring_function(string, id, size, lineHeight = DEFAULT_LINE_HEIGHT, letterSpacing = 0) {
    const trueSize = size * GLOBAL_FONT_SCALING_FACTOR;
    const el = getTextMeasureElement();
    el.style.fontFamily = `font${id}`;
    el.style.fontSize = `${trueSize}px`;
    const effectiveLineHeight = lineHeight != null && lineHeight > 0 ? lineHeight : DEFAULT_LINE_HEIGHT;
    const effectiveLetterSpacing = letterSpacing != null ? letterSpacing : 0;
    el.style.lineHeight = String(effectiveLineHeight);
    el.style.letterSpacing = `${effectiveLetterSpacing}px`;
    const isEmpty = !string || string.length === 0;
    const hasNewlines = !isEmpty && string.includes("\n");

    if (hasNewlines) {
        el.style.whiteSpace = "pre-wrap";
        el.style.wordBreak = "break-word";
        el.style.overflowWrap = "anywhere";
        el.textContent = string;
        const rect = el.getBoundingClientRect();
        return [Math.ceil(rect.width), Math.ceil(rect.height) || Math.ceil(trueSize * effectiveLineHeight)];
    }

    el.style.whiteSpace = "pre";
    el.style.wordBreak = "normal";
    el.style.overflowWrap = "normal";
    el.textContent = isEmpty ? "\u200B" : string;
    const rect = el.getBoundingClientRect();
    const width = isEmpty ? 0 : Math.ceil(rect.width);
    const height = Math.ceil(rect.height) || Math.ceil(trueSize);
    return [width, height];
}

// Get mouse position as [x, y]
let mouseX = 0;
let mouseY = 0;
window.addEventListener("mousemove", (e) => {
    mouseX = e.clientX;
    mouseY = e.clientY;
});
export function get_mouse_position() {
    return [mouseX, mouseY];
}

// Track currently pressed keys
let keysPressed = new Set();
let keyQueue = [];
window.addEventListener("keydown", (e) => {
    keysPressed.add(e.code);
});
window.addEventListener("keyup", (e) => {
    keysPressed.delete(e.code);
    let char = e.key;
    if (char.length === 1) {
        keyQueue.push(char);
    }
});

export function keyDown(keyCode) {
    return keysPressed.has(keyCode);
}

export function readChar() {
    if (keyQueue.length > 0) return keyQueue.shift();
    return null;
}

export function hasInput() {
    return keyQueue.length > 0;
}

// Check if a mouse button is down
let mouseButtons = new Set();
window.addEventListener("mousedown", (e) => mouseButtons.add(e.button));
window.addEventListener("mouseup", (e) => mouseButtons.delete(e.button));

export function mouse_button_down(n) {
    return mouseButtons.has(n);
}

export function getPressedKeys() {
    return Array.from(keysPressed);
}

// Get vertical mouse wheel movement
let wheelDelta = 0;
window.addEventListener("wheel", (e) => {
    wheelDelta = e.deltaY;
});
export function get_mouse_wheel_move_v() {
    const delta = wheelDelta;
    wheelDelta = 0;
    return delta;
}

// Root container for all rendered elements
let renderRoot = null;

const COTIS_ID_PREFIX = "cotis-";

// Scissor (clipping) stack
let scissorStack = [];
let customElementsById = new Map();
// Retained custom elements: Rust id -> { el, lastHtml }. Reused across frames so inputs etc. keep state; recreated only when html changes or command is removed.
let customElementRetained = new Map();

/** Creates #cotis-render-root once; required before any DOM draw (Rust path uses beginFrame first). */
function ensureRenderRoot() {
    if (renderRoot) return;
    if (!document.body) {
        throw new Error(
            "web-cotis: document.body is not available yet. Start the WASM app after the DOM is ready (defer the script or wait for DOMContentLoaded)."
        );
    }
    console.log("Initing HTML renderer");

    renderRoot = document.createElement("div");
    renderRoot.id = "cotis-render-root";
    renderRoot.style.cssText = `
        position: fixed;
        top: 0; left: 0;
        width: 100%; height: 100%;
        overflow: hidden;
    `;
    document.body.style.margin = "0";
    document.body.style.padding = "0";
    document.body.style.overflow = "hidden";
    document.body.appendChild(renderRoot);
}

export function init_html_root() {
    ensureRenderRoot();
}

const GLOBAL_FONT_SCALING_FACTOR = 1;

export async function loadFont(fontId, fontUrl) {
    const font = new FontFace(`font${fontId}`, `url(${fontUrl})`);
    await font.load();
    document.fonts.add(font);
}

// --- ID-based get-or-create and frame lifecycle ---

function elementId(renderCommand) {
    return `${COTIS_ID_PREFIX}${renderCommand.id ?? 0}`;
}

function getOrCreateElement(id) {
    const el = renderRoot.querySelector(`[id="${CSS.escape(id)}"]`);
    if (el) return el;
    const newEl = document.createElement("div");
    newEl.id = id;
    return newEl;
}

export function beginFrame() {
    ensureRenderRoot();
    scissorStack = [];
    customElementsById.clear();
}

export function endFrame(usedElementIds) {
    if (!renderRoot) return;
    const cotisElements = renderRoot.querySelectorAll(`[id^="${COTIS_ID_PREFIX}"]`);
    for (const el of cotisElements) {
        if (!usedElementIds.has(el.id)) {
            if (el.parentNode) el.parentNode.removeChild(el);
        }
    }
    for (const [rustId, { el }] of customElementRetained.entries()) {
        if (!usedElementIds.has(el.id)) {
            customElementRetained.delete(rustId);
            customElementsById.delete(rustId);
        }
    }
}

// Returns the current clip container (innermost scissor div, or renderRoot)
function currentContainer() {
    if (scissorStack.length > 0) {
        return scissorStack[scissorStack.length - 1].container;
    }
    return renderRoot;
}

// Global offset (viewport-space) of the current container.
// Root container is pinned at 0,0; nested scissor containers keep their
// own global origin so descendants can be converted to local coordinates.
function currentContainerOffset() {
    if (scissorStack.length > 0) {
        const top = scissorStack[scissorStack.length - 1];
        return { x: top.globalX, y: top.globalY };
    }
    return { x: 0, y: 0 };
}

function toLocalPosition(bb) {
    const offset = currentContainerOffset();
    return {
        left: bb.x - offset.x,
        top: bb.y - offset.y
    };
}

// Append element to the current scissor container only when new or in wrong container (avoids moving existing elements every frame)
export function appendToCurrentContainer(el) {
    ensureRenderRoot();
    const container = currentContainer();
    if (!container) return;
    if (el.parentNode !== container) {
        container.appendChild(el);
    }
}

// Push a new scissor container onto the stack
export function scissorStackPush(container, globalX, globalY) {
    scissorStack.push({ container, globalX, globalY });
}

// Pop a scissor container from the stack
export function scissorStackPop() {
    if (scissorStack.length > 0) {
        scissorStack.pop();
    }
}

function applyStyle(el, styles) {
    for (const [prop, value] of Object.entries(styles)) {
        el.style[prop] = value;
    }
}

// Accept alpha from either [0..255] or [0..1] payloads.
function toCssAlpha(a) {
    const n = Number(a);
    if (!Number.isFinite(n)) return 1;
    if (n > 1) return Math.max(0, Math.min(1, n / 255));
    return Math.max(0, Math.min(1, n));
}

function asAttachmentPercent(attachmentPoint) {
    switch (String(attachmentPoint || "")) {
        case "TopLeft": return { x: "0%", y: "0%" };
        case "TopCenter": return { x: "50%", y: "0%" };
        case "TopRight": return { x: "100%", y: "0%" };
        case "CenterLeft": return { x: "0%", y: "50%" };
        case "CenterCenter": return { x: "50%", y: "50%" };
        case "CenterRight": return { x: "100%", y: "50%" };
        case "BottomLeft": return { x: "0%", y: "100%" };
        case "BottomCenter": return { x: "50%", y: "100%" };
        case "BottomRight": return { x: "100%", y: "100%" };
        default: return { x: "50%", y: "50%" };
    }
}

function cssColor(color) {
    if (!color || typeof color !== "object") return "rgba(0,0,0,1)";
    return `rgba(${Number(color.r) || 0},${Number(color.g) || 0},${Number(color.b) || 0},${toCssAlpha(color.a)})`;
}

function cssLinearGradient(layer) {
    const startAttach = asAttachmentPercent(layer?.start?.attachmentPoint);
    const endAttach = asAttachmentPercent(layer?.end?.attachmentPoint);
    const sx = Number(layer?.start?.x);
    const sy = Number(layer?.start?.y);
    const ex = Number(layer?.end?.x);
    const ey = Number(layer?.end?.y);
    const startXNum = Number.isFinite(sx) ? sx : (Number.parseFloat(startAttach.x) / 100);
    const startYNum = Number.isFinite(sy) ? sy : (Number.parseFloat(startAttach.y) / 100);
    const endXNum = Number.isFinite(ex) ? ex : (Number.parseFloat(endAttach.x) / 100);
    const endYNum = Number.isFinite(ey) ? ey : (Number.parseFloat(endAttach.y) / 100);
    const dx = endXNum - startXNum;
    const dy = endYNum - startYNum;
    const angleDeg = Math.atan2(dy, dx) * (180 / Math.PI) + 90;
    const stops = Array.isArray(layer?.stops)
        ? layer.stops.map((s) => `${cssColor(s?.color)} ${(Number(s?.offset) || 0) * 100}%`).join(", ")
        : `${cssColor({ r: 0, g: 0, b: 0, a: 255 })} 0%, ${cssColor({ r: 0, g: 0, b: 0, a: 255 })} 100%`;
    return `linear-gradient(${angleDeg}deg, ${stops})`;
}

function cssRadialGradient(layer) {
    const centerAttach = asAttachmentPercent(layer?.center?.attachmentPoint);
    const cx = Number(layer?.center?.x);
    const cy = Number(layer?.center?.y);
    const centerX = Number.isFinite(cx) ? `${cx * 100}%` : centerAttach.x;
    const centerY = Number.isFinite(cy) ? `${cy * 100}%` : centerAttach.y;
    const stops = Array.isArray(layer?.stops)
        ? layer.stops.map((s) => `${cssColor(s?.color)} ${(Number(s?.offset) || 0) * 100}%`).join(", ")
        : `${cssColor({ r: 0, g: 0, b: 0, a: 255 })} 0%, ${cssColor({ r: 0, g: 0, b: 0, a: 255 })} 100%`;
    return `radial-gradient(circle at ${centerX} ${centerY}, ${stops})`;
}

function flattenColorLayers(payload) {
    if (!payload || typeof payload !== "object") return [];
    if (payload.type === "Layered" && Array.isArray(payload.layers)) {
        return payload.layers.flatMap(flattenColorLayers);
    }
    return [payload];
}

function colorPayloadToStyle(payload) {
    // Backward compatibility with old flat rgba payload.
    if (payload && payload.r != null && payload.g != null && payload.b != null) {
        return {
            color: cssColor(payload),
            image: "",
        };
    }

    const layers = flattenColorLayers(payload);
    if (layers.length === 0) {
        return { color: "rgba(0,0,0,0)", image: "" };
    }

    const cssLayers = [];
    let fallbackColor = "rgba(0,0,0,0)";
    for (const layer of layers) {
        if (!layer || typeof layer !== "object") continue;
        if (layer.type === "Solid") {
            const c = cssColor(layer.color);
            cssLayers.push(`linear-gradient(${c}, ${c})`);
            fallbackColor = c;
        } else if (layer.type === "Linear") {
            cssLayers.push(cssLinearGradient(layer));
        } else if (layer.type === "Radial") {
            cssLayers.push(cssRadialGradient(layer));
        }
    }
    return {
        color: fallbackColor,
        image: cssLayers.join(", "),
    };
}

/** @param {unknown} tag */
function normalizeCustomHostTag(tag) {
    const t = tag == null ? "div" : String(tag).trim().toLowerCase();
    if (t === "button") return "button";
    return "div";
}

/** @param {Record<string, unknown> | null | undefined} config */
function hostTagAndExtraFromConfig(config) {
    if (!config) {
        return { tagName: "div", extraStyle: null };
    }
    const tagName = normalizeCustomHostTag(config.tag);
    const extraStyle =
        config.extraStyle != null && String(config.extraStyle).length > 0
            ? String(config.extraStyle)
            : null;
    return { tagName, extraStyle };
}

/**
 * Host DOM element for commands that carry `tag` / `extraStyle` from Rust `extra_data`.
 * Recreates the node when tag or extraStyle changes.
 * @param {string} id
 * @param {string} tagName
 * @param {string | null} extraStyle
 * @param {(el: HTMLElement) => void} applyBase
 */
export function getOrCreateHostElement(id, tagName, extraStyle, applyBase) {
    ensureRenderRoot();
    const signature = `${tagName}\u0000${extraStyle ?? ""}`;
    const existing = renderRoot.querySelector(`[id="${CSS.escape(id)}"]`);
    if (
        existing &&
        existing.tagName.toLowerCase() === tagName &&
        existing.dataset.cotisHostSig === signature
    ) {
        applyBase(existing);
        return existing;
    }
    if (existing && existing.parentNode) {
        existing.parentNode.removeChild(existing);
    }
    const el = document.createElement(tagName);
    el.id = id;
    applyBase(el);
    if (extraStyle) {
        el.style.cssText += `;${extraStyle}`;
    }
    el.dataset.cotisHostSig = signature;
    return el;
}

// Avoid assigning el.textContent every frame: that replaces child nodes and clears selection.
// When the element is a single Text node, update CharacterData.data in place instead.
function setTextContentPreserveSelection(el, text) {
    const s = text == null ? "" : String(text);
    const children = el.childNodes;
    if (children.length === 1 && children[0].nodeType === Node.TEXT_NODE) {
        const tn = children[0];
        if (tn.data === s) return;
        tn.data = s;
        return;
    }
    if (el.textContent === s) return;
    el.textContent = s;
}

function collectElementProperties(element) {
    if (!element) {
        return null;
    }

    const attrs = {};
    for (const attr of element.attributes) {
        attrs[attr.name] = attr.value;
    }

    const dataset = {};
    for (const key in element.dataset) {
        if (Object.prototype.hasOwnProperty.call(element.dataset, key)) {
            dataset[key] = element.dataset[key];
        }
    }

    const rect = element.getBoundingClientRect();
    return {
        tagName: element.tagName.toLowerCase(),
        id: element.id || "",
        name: element.name || "",
        type: element.type || "",
        value: "value" in element ? element.value : "",
        placeholder: element.placeholder || "",
        checked: Boolean(element.checked),
        disabled: Boolean(element.disabled),
        readOnly: Boolean(element.readOnly),
        className: element.className || "",
        textContent: element.textContent || "",
        innerHTML: element.innerHTML || "",
        attributes: attrs,
        dataset,
        boundingBox: {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height
        }
    };
}

export function get_custom_element_html(elementId) {
    const element = customElementsById.get(String(elementId));
    if (!element) {
        return null;
    }
    return element.innerHTML;
}

export function get_custom_element_properties(elementId, selector) {
    const host = customElementsById.get(String(elementId));
    if (!host) {
        return null;
    }

    let target = host;
    if (selector && selector.length > 0) {
        target = host.querySelector(selector);
    } else {
        target = host.querySelector("input, textarea, select, button") || host.firstElementChild || host;
    }

    const props = collectElementProperties(target);
    if (!props) {
        return null;
    }
    return JSON.stringify(props);
}

// --- Command constants ---
const COTIS_RENDER_COMMAND_TYPE_NONE          = "None";
const COTIS_RENDER_COMMAND_TYPE_RECTANGLE     = "Rectangle";
const COTIS_RENDER_COMMAND_TYPE_BORDER        = "Border";
const COTIS_RENDER_COMMAND_TYPE_TEXT          = "Text";
const COTIS_RENDER_COMMAND_TYPE_IMAGE         = "Image";
const COTIS_RENDER_COMMAND_TYPE_SCISSOR_START = "ScissorStart";
const COTIS_RENDER_COMMAND_TYPE_SCISSOR_END   = "ScissorEnd";
const COTIS_RENDER_COMMAND_TYPE_CUSTOM        = "Custom";

// --- Main render loop ---

function renderLoopHTML(commands) {
    beginFrame();
    const usedElementIds = new Set();

    for (let i = 0; i < commands.length; i++) {
        const renderCommand = JSON.parse(commands[i]);
        const bb = renderCommand.boundingBox;

        switch (renderCommand.commandType) {

            case COTIS_RENDER_COMMAND_TYPE_NONE: {
                break;
            }

            case COTIS_RENDER_COMMAND_TYPE_RECTANGLE: {
                const id = elementId(renderCommand);
                const config = renderCommand.config;
                const { tagName, extraStyle } = hostTagAndExtraFromConfig(config);
                const bgStyle = colorPayloadToStyle(config.backgroundColor);
                const cr     = config.cornerRadius;
                const pos = toLocalPosition(bb);

                const el = getOrCreateHostElement(id, tagName, extraStyle, (el) => {
                    applyStyle(el, {
                        position: "absolute",
                        left: `${pos.left}px`,
                        top: `${pos.top}px`,
                        width: `${bb.width}px`,
                        height: `${bb.height}px`,
                        backgroundColor: bgStyle.color,
                        backgroundImage: bgStyle.image,
                        backgroundSize: "",
                        backgroundRepeat: "",
                        backgroundPosition: "",
                        borderRadius: `${cr.topLeft}px ${cr.topRight}px ${cr.bottomRight}px ${cr.bottomLeft}px`,
                        boxSizing: "border-box",
                        pointerEvents: "auto"
                    });
                });
                appendToCurrentContainer(el);
                usedElementIds.add(id);
                break;
            }

            case COTIS_RENDER_COMMAND_TYPE_BORDER: {
                const id = elementId(renderCommand);
                const config = renderCommand.config;
                const { tagName, extraStyle } = hostTagAndExtraFromConfig(config);
                const color  = config.color;
                const cr     = config.cornerRadius;
                const w      = config.width;
                const pos = toLocalPosition(bb);

                const el = getOrCreateHostElement(id, tagName, extraStyle, (el) => {
                    applyStyle(el, {
                        position: "absolute",
                        left: `${pos.left}px`,
                        top: `${pos.top}px`,
                        width: `${bb.width}px`,
                        height: `${bb.height}px`,
                        backgroundImage: "",
                        backgroundSize: "",
                        backgroundRepeat: "",
                        backgroundPosition: "",
                        borderTop: `${w.top}px solid rgba(${color.r},${color.g},${color.b},${toCssAlpha(color.a)})`,
                        borderRight: `${w.right}px solid rgba(${color.r},${color.g},${color.b},${toCssAlpha(color.a)})`,
                        borderBottom: `${w.bottom}px solid rgba(${color.r},${color.g},${color.b},${toCssAlpha(color.a)})`,
                        borderLeft: `${w.left}px solid rgba(${color.r},${color.g},${color.b},${toCssAlpha(color.a)})`,
                        borderRadius: `${cr.topLeft}px ${cr.topRight}px ${cr.bottomRight}px ${cr.bottomLeft}px`,
                        boxSizing: "border-box",
                        pointerEvents: "auto"
                    });
                });
                appendToCurrentContainer(el);
                usedElementIds.add(id);
                break;
            }

            case COTIS_RENDER_COMMAND_TYPE_TEXT: {
                const id = elementId(renderCommand);
                const config   = renderCommand.config;
                const { tagName, extraStyle } = hostTagAndExtraFromConfig(config);
                const textStyle = colorPayloadToStyle(config.color);
                const fontSize = config.fontSize * GLOBAL_FONT_SCALING_FACTOR;
                const lineHeight = config.lineHeight != null && config.lineHeight > 0 ? config.lineHeight : DEFAULT_LINE_HEIGHT;
                const letterSpacing = config.letterSpacing != null ? config.letterSpacing : 0;
                const pos = toLocalPosition(bb);

                const el = getOrCreateHostElement(id, tagName, extraStyle, (el) => {
                    setTextContentPreserveSelection(el, config.text);
                    applyStyle(el, {
                        position: "absolute",
                        left: `${pos.left}px`,
                        top: `${pos.top}px`,
                        width: `${bb.width}px`,
                        height: `${bb.height}px`,
                        backgroundImage: textStyle.image,
                        backgroundSize: "",
                        backgroundRepeat: "",
                        backgroundPosition: "",
                        fontFamily: `font${config.fontId}`,
                        fontSize: `${fontSize}px`,
                        lineHeight: String(lineHeight),
                        letterSpacing: `${letterSpacing}px`,
                        color: textStyle.image ? "transparent" : textStyle.color,
                        backgroundClip: textStyle.image ? "text" : "",
                        WebkitBackgroundClip: textStyle.image ? "text" : "",
                        display: "flex",
                        alignItems: "flex-start",
                        whiteSpace: "pre-wrap",
                        wordBreak: "break-word",
                        overflowWrap: "anywhere",
                        overflow: "hidden",
                        boxSizing: "border-box",
                        pointerEvents: "auto",
                        userSelect: "text",
                    });
                });
                appendToCurrentContainer(el);
                usedElementIds.add(id);
                break;
            }

            case COTIS_RENDER_COMMAND_TYPE_SCISSOR_START: {
                const id = elementId(renderCommand);
                const container = getOrCreateElement(id);
                const config = renderCommand.config;
                const cr     = config.cornerRadius || {};
                const pos = toLocalPosition(bb);

                applyStyle(container, {
                    position: "absolute",
                    left: `${pos.left}px`,
                    top: `${pos.top}px`,
                    width: `${bb.width}px`,
                    height: `${bb.height}px`,
                    overflow: "hidden",
                    borderRadius: `${cr.topLeft || 0}px ${cr.topRight || 0}px ${cr.bottomRight || 0}px ${cr.bottomLeft || 0}px`,
                    boxSizing: "border-box"
                });
                appendToCurrentContainer(container);
                scissorStack.push({ container, globalX: bb.x, globalY: bb.y });
                usedElementIds.add(id);
                break;
            }

            case COTIS_RENDER_COMMAND_TYPE_SCISSOR_END: {
                if (scissorStack.length > 0) scissorStack.pop();
                break;
            }

            case COTIS_RENDER_COMMAND_TYPE_IMAGE: {
                const id = elementId(renderCommand);
                const config   = renderCommand.config;
                const { tagName, extraStyle } = hostTagAndExtraFromConfig(config);
                const imageURL = config.url;
                const pos = toLocalPosition(bb);

                const el = getOrCreateHostElement(id, tagName, extraStyle, (el) => {
                    applyStyle(el, {
                        position: "absolute",
                        left: `${pos.left}px`,
                        top: `${pos.top}px`,
                        width: `${bb.width}px`,
                        height: `${bb.height}px`,
                        backgroundImage: `url("${imageURL}")`,
                        backgroundSize: "100% 100%",
                        backgroundRepeat: "no-repeat",
                        backgroundPosition: "center",
                        display: "block",
                        pointerEvents: "auto"
                    });
                });
                appendToCurrentContainer(el);
                usedElementIds.add(id);
                break;
            }

            case COTIS_RENDER_COMMAND_TYPE_CUSTOM: {
                const config = renderCommand.config;
                const html = config?.html;
                if (!html) {
                    break;
                }

                const rustId = String(renderCommand.id ?? 0);
                const id = elementId(renderCommand);
                const pos = toLocalPosition(bb);
                const tagName = normalizeCustomHostTag(config.tag);
                const extraStyle =
                    config.extraStyle != null && String(config.extraStyle).length > 0
                        ? String(config.extraStyle)
                        : null;

                const styleMap = {
                    position: "absolute",
                    left: `${pos.left}px`,
                    top: `${pos.top}px`,
                    width: `${bb.width}px`,
                    height: `${bb.height}px`,
                    backgroundImage: "",
                    backgroundSize: "",
                    backgroundRepeat: "",
                    backgroundPosition: "",
                    boxSizing: "border-box",
                    pointerEvents: "auto"
                };

                const retained = customElementRetained.get(rustId);
                const canReuse =
                    retained &&
                    retained.lastHtml === html &&
                    retained.tagName === tagName &&
                    retained.extraStyle === extraStyle;

                let el;
                if (canReuse) {
                    el = retained.el;
                    applyStyle(el, styleMap);
                } else {
                    if (retained && retained.el.parentNode) {
                        retained.el.parentNode.removeChild(retained.el);
                    }
                    el = document.createElement(tagName);
                    el.id = id;
                    applyStyle(el, styleMap);
                    if (extraStyle) {
                        el.style.cssText += `;${extraStyle}`;
                    }
                    el.innerHTML = html;
                    customElementRetained.set(rustId, {
                        el,
                        lastHtml: html,
                        tagName,
                        extraStyle
                    });
                }

                customElementsById.set(rustId, el);
                appendToCurrentContainer(el);
                usedElementIds.add(id);
                break;
            }
        }
    }

    endFrame(usedElementIds);
}