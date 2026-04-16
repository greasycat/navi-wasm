import { statusPanel } from "./dom.js";

export function setStatus(title, body, kind = "info") {
  statusPanel.className = `status-panel status-${kind}`;
  statusPanel.innerHTML = `
    <p class="status-title">${title}</p>
    <p class="status-body">${body}</p>
  `;
}
