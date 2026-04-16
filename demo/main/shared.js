export function cloneSpec(value) {
  if (globalThis.structuredClone) {
    return globalThis.structuredClone(value);
  }

  return JSON.parse(JSON.stringify(value));
}

export function setCanvasDragging(canvas, dragging) {
  canvas.classList.toggle("is-dragging", dragging);
}

export function eventPoint(event, canvas) {
  const rect = canvas.getBoundingClientRect();
  const scaleX = canvas.width / rect.width;
  const scaleY = canvas.height / rect.height;
  return {
    x: (event.clientX - rect.left) * scaleX,
    y: (event.clientY - rect.top) * scaleY
  };
}

export function selectionName(item, fallback) {
  return item?.name ?? item?.label ?? fallback;
}

export function propertyEntries(properties) {
  if (!properties) {
    return [];
  }

  if (properties instanceof Map) {
    return Array.from(properties.entries());
  }

  return Object.entries(properties);
}

export function renderProperties(properties) {
  const entries = propertyEntries(properties);
  if (!entries.length) {
    return `<p class="details-empty-copy">No custom properties on this selection.</p>`;
  }

  return `
    <dl class="property-list">
      ${entries
        .map(
          ([key, value]) => `
            <div class="property-row">
              <dt>${key}</dt>
              <dd>${value}</dd>
            </div>
          `
        )
        .join("")}
    </dl>
  `;
}

export function buildPropertyList(properties) {
  const entries = propertyEntries(properties);
  if (!entries.length) {
    const p = document.createElement("p");
    p.className = "details-empty-copy";
    p.textContent = "No custom properties on this selection.";
    return p;
  }

  const dl = document.createElement("dl");
  dl.className = "property-list";
  for (const [key, value] of entries) {
    const row = document.createElement("div");
    row.className = "property-row";
    const dt = document.createElement("dt");
    dt.textContent = key;
    const dd = document.createElement("dd");
    dd.textContent = value;
    row.append(dt, dd);
    dl.append(row);
  }
  return dl;
}

export function normalizeViewportOffsets(spec) {
  if (!spec || typeof spec !== "object") {
    return;
  }

  if (Object.hasOwn(spec, "offset_x")) {
    spec.offset_x = Number.isFinite(spec.offset_x) ? Math.round(spec.offset_x) : 0;
  }

  if (Object.hasOwn(spec, "offset_y")) {
    spec.offset_y = Number.isFinite(spec.offset_y) ? Math.round(spec.offset_y) : 0;
  }
}
