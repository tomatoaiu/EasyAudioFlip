const { invoke } = window.__TAURI__.core;
const { getCurrentWindow } = window.__TAURI__.window;

function render(devices) {
  const list = document.getElementById("device-list");
  list.innerHTML = "";

  for (const d of devices) {
    const row = document.createElement("div");
    row.className = "device-row";

    const checkbox = document.createElement("input");
    checkbox.type = "checkbox";
    checkbox.checked = d.enabled;

    const name = document.createElement("span");
    name.className = "device-name";
    name.textContent = d.name;

    row.appendChild(checkbox);
    row.appendChild(name);

    if (d.is_current) {
      const marker = document.createElement("span");
      marker.className = "current-marker";
      marker.textContent = "\u25B6";
      row.appendChild(marker);
    }

    row.addEventListener("click", async (e) => {
      e.preventDefault();
      const updated = await invoke("toggle_panel_device", { deviceId: d.id });
      render(updated);
    });

    list.appendChild(row);
  }
}

document.addEventListener("DOMContentLoaded", async () => {
  const devices = await invoke("get_panel_devices");
  render(devices);

  document.getElementById("quit").addEventListener("click", () => {
    invoke("quit_app");
  });

  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape") {
      getCurrentWindow().hide();
    }
  });

  // Re-fetch data every time panel becomes visible
  window.addEventListener("focus", async () => {
    const devices = await invoke("get_panel_devices");
    render(devices);
  });
});
