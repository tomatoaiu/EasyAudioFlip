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
      row.classList.add("current");
      const marker = document.createElement("span");
      marker.className = "current-marker";
      marker.textContent = "\u25B6";
      row.appendChild(marker);
    }

    checkbox.addEventListener("click", async (e) => {
      e.preventDefault();
      e.stopPropagation();
      const updated = await invoke("toggle_panel_device", { deviceId: d.id });
      render(updated);
    });

    let clickTimer = null;
    row.addEventListener("click", (e) => {
      if (row.classList.contains("editing") || e.target === checkbox) {
        return;
      }
      e.preventDefault();
      if (e.detail > 1) {
        return;
      }
      clearTimeout(clickTimer);
      clickTimer = setTimeout(async () => {
        const updated = await invoke("select_panel_device", { deviceId: d.id });
        render(updated);
      }, 220);
    });

    row.addEventListener("dblclick", (e) => {
      if (e.target === checkbox) {
        return;
      }
      e.preventDefault();
      e.stopPropagation();
      clearTimeout(clickTimer);
      beginRename(row, name, d, devices);
    });

    list.appendChild(row);
  }
}

function beginRename(row, name, device, devices) {
  row.classList.add("editing");
  const input = document.createElement("input");
  input.type = "text";
  input.className = "device-name-input";
  input.value = device.name;
  name.replaceWith(input);
  input.focus();
  input.select();

  let finished = false;
  const cancel = () => {
    if (finished) {
      return;
    }
    finished = true;
    render(devices);
  };

  input.addEventListener("keydown", async (e) => {
    if (e.key === "Enter") {
      e.preventDefault();
      e.stopPropagation();
      if (finished) {
        return;
      }
      finished = true;
      try {
        const updated = await invoke("rename_panel_device", {
          deviceId: device.id,
          name: input.value,
        });
        render(updated);
      } catch (err) {
        console.error("Failed to rename device:", err);
        render(devices);
      }
    } else if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      cancel();
    }
  });

  input.addEventListener("blur", cancel);
}

document.addEventListener("DOMContentLoaded", async () => {
  const devices = await invoke("get_panel_devices");
  render(devices);

  document.getElementById("quit").addEventListener("click", () => {
    invoke("quit_app");
  });

  // Autostart toggle
  const autostartToggle = document.getElementById("autostart-toggle");

  async function loadAutostartState() {
    try {
      autostartToggle.checked = await invoke("is_autostart_enabled");
    } catch (err) {
      console.error("Failed to load autostart state:", err);
    }
  }

  autostartToggle.addEventListener("change", async (e) => {
    const shouldEnable = e.target.checked;
    try {
      await invoke(shouldEnable ? "enable_autostart" : "disable_autostart");
    } catch (err) {
      console.error("Failed to toggle autostart:", err);
      autostartToggle.checked = !shouldEnable;
    }
  });

  await loadAutostartState();

  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape") {
      getCurrentWindow().hide();
    }
  });

  // Re-fetch data every time panel becomes visible
  window.addEventListener("focus", async () => {
    const devices = await invoke("get_panel_devices");
    render(devices);
    await loadAutostartState();
  });
});
