// form.js for esp32example

document.addEventListener("DOMContentLoaded", function () {
    bindForm("examplemsg", handleSampleSubmit);
    bindForm("esp32cfg", handleCfgSubmit);
    bindForm("esp32fw", handleFwSubmit);
    initUptime();
    initDeviceState();
    initSampleState();
    initDetectedSensors();
    initTemperatures();
});

function bindForm(name, handler) {
    const form = document.querySelector(`form[name='${name}']`);
    if (!form) return;
    ensureStatusNode(form);
    form.addEventListener("submit", handler);
}

function ensureStatusNode(form) {
    let status = form.querySelector(".form-status");
    if (status) return status;

    status = document.createElement("div");
    status.className = "form-status";
    status.setAttribute("role", "status");
    status.setAttribute("aria-live", "polite");
    status.hidden = true;
    form.appendChild(status);
    return status;
}

function setFormStatus(form, kind, message) {
    const status = ensureStatusNode(form);
    status.hidden = !message;
    status.className = `form-status${kind ? ` is-${kind}` : ""}`;
    status.textContent = message || "";
}

function setFormBusy(form, busy, busyLabel) {
    const submit = form.querySelector("input[type='submit']");
    if (!submit) return;

    if (busy) {
        if (!submit.dataset.label) submit.dataset.label = submit.value;
        submit.disabled = true;
        submit.value = busyLabel || "Working...";
    } else {
        submit.disabled = false;
        if (submit.dataset.label) submit.value = submit.dataset.label;
    }
}

async function fetchPayloadOrError(url, options) {
    const response = await fetch(url, options);
    const contentType = response.headers.get("content-type") || "";

    let payload;
    if (contentType.includes("application/json")) {
        payload = await response.json();
    } else {
        const text = await response.text();
        payload = {message: (text || "").trim()};
    }

    if (!response.ok) {
        throw new Error(payload.message || `Request failed (${response.status})`);
    }
    return payload;
}

async function updateUptime() {
    const node = document.getElementById("uptime");
    if (!node) return;

    try {
        const response = await fetch("/uptime");
        const json = await response.json();
        node.textContent = `Uptime: ${json.uptime} (${json.uptime_s})`;
    } catch (_error) {
        node.textContent = "Uptime unavailable";
    }
}

function initUptime() {
    if (!document.getElementById("uptime")) return;
    updateUptime();
    window.setInterval(updateUptime, 10e3);
}

async function updateDeviceState() {
    const node = document.getElementById("device-state");
    if (!node) return;

    try {
        const response = await fetch("/state");
        const json = await response.json();

        node.innerHTML = `
            <table>
                <tr><th>Device ID</th><td><code>${json.device_id}</code></td></tr>
                <tr><th>IP address</th><td><code>${json.ip_addr}</code></td></tr>
                <tr><th>MAC</th><td><code>${json.mac_addr}</code></td></tr>
                <tr><th>WiFi</th><td>${json.wifi_up ? "up" : "down"}${json.ap_mode ? " (AP mode)" : ""}</td></tr>
                <tr><th>NTP</th><td>${json.ntp_ok ? "synced" : "not synced"}</td></tr>
                <tr><th>MQTT</th><td>${json.mqtt_enabled ? "enabled" : "disabled"}</td></tr>
                <tr><th>DS18B20</th><td>${json.sensor_enabled ? "enabled" : "disabled"} (${json.sensor_count} detected)</td></tr>
                <tr><th>OTA slot</th><td>${json.ota_slot}</td></tr>
            </table>
        `;
    } catch (_error) {
        node.textContent = "Device state unavailable";
    }
}

function initDeviceState() {
    if (!document.getElementById("device-state")) return;
    updateDeviceState();
    window.setInterval(updateDeviceState, 10e3);
}

async function updateSampleState() {
    const node = document.getElementById("sample-state");
    if (!node) return;

    try {
        const response = await fetch("/sample");
        const json = await response.json();
        node.innerHTML = `
            <table>
                <tr><th>Counter</th><td>${json.counter}</td></tr>
                <tr><th>Source</th><td>${json.source}</td></tr>
                <tr><th>Message</th><td>${json.message}</td></tr>
                <tr><th>Updated at</th><td><code>${json.updated_at}</code></td></tr>
            </table>
        `;
    } catch (_error) {
        node.textContent = "Sample state unavailable";
    }
}

function initSampleState() {
    if (!document.getElementById("sample-state")) return;
    updateSampleState();
    window.setInterval(updateSampleState, 10e3);
}

async function updateDetectedSensors() {
    const node = document.getElementById("detected-sensors");
    if (!node) return;

    try {
        const response = await fetch("/sensors");
        const json = await response.json();

        if (!json.sensors.length) {
            node.innerHTML = '<div class="table-meta">No DS18B20 sensors detected at boot</div>';
            return;
        }

        let rows = "<tr><th>IO pin</th><th>Sensor</th></tr>\n";
        json.sensors.forEach((sensor) => {
            rows += `<tr><td><code>${sensor.iopin}</code></td><td>${sensor.sensor}</td></tr>\n`;
        });
        node.innerHTML =
            `<div class="table-meta">Detected at boot: <b>${json.sensors.length}</b></div>` +
            `<table>${rows}</table>`;
    } catch (_error) {
        node.textContent = "Sensor inventory unavailable";
    }
}

function initDetectedSensors() {
    if (!document.getElementById("detected-sensors")) return;
    updateDetectedSensors();
}

async function updateTemperatures() {
    const node = document.getElementById("temperatures");
    if (!node) return;

    try {
        const response = await fetch("/temp");
        const json = await response.json();
        let rows = "<tr><th>IO pin</th><th>Sensor</th><th>Value (C)</th></tr>\n";
        json.temperatures.forEach((temp) => {
            rows += `<tr><td><code>${temp.iopin}</code></td><td>${temp.sensor}</td><td class="temperature-value">${temp.value}</td></tr>\n`;
        });
        node.innerHTML =
            `<div class="table-meta">Last update: <b>${json.last_update}</b></div>` +
            `<table>${rows}</table>`;
    } catch (_error) {
        node.textContent = "Temperature data unavailable";
    }
}

function initTemperatures() {
    if (!document.getElementById("temperatures")) return;
    updateTemperatures();
    window.setInterval(updateTemperatures, 60e3);
}

const handleSampleSubmit = async (event) => {
    event.preventDefault();
    const form = event.currentTarget;
    const url = form.action;

    setFormBusy(form, true, "Sending...");
    setFormStatus(form, "busy", "Updating sample state...");
    try {
        const formData = new FormData(form);
        const responseData = await postSampleAsJson({url, formData});
        setFormStatus(form, "ok", responseData.message || "Sample state updated");
        updateSampleState();
        updateDeviceState();
    } catch (error) {
        console.error(error);
        setFormStatus(form, "error", error.message || "Sample update failed");
    } finally {
        setFormBusy(form, false);
    }
};

const handleCfgSubmit = async (event) => {
    event.preventDefault();
    const form = event.currentTarget;
    const url = form.action;

    setFormBusy(form, true, "Saving...");
    setFormStatus(form, "busy", "Saving config...");
    try {
        const formData = new FormData(form);
        const responseData = await postCfgDataAsJson({url, formData});
        setFormStatus(form, "ok", responseData.message || "Config saved, device will reboot");
    } catch (error) {
        console.error(error);
        setFormStatus(form, "error", error.message || "Config save failed");
    } finally {
        setFormBusy(form, false);
    }
};

const handleFwSubmit = async (event) => {
    event.preventDefault();
    const form = event.currentTarget;
    const url = form.action;
    const firmwareUrl = String(new FormData(form).get("url") || "").trim();

    try {
        const parsed = new URL(firmwareUrl);
        if (parsed.protocol !== "http:" && parsed.protocol !== "https:") {
            throw new Error("Firmware URL must start with http:// or https://");
        }
    } catch (_error) {
        setFormStatus(form, "error", "Firmware URL must start with http:// or https://");
        return;
    }

    if (!window.confirm("Start firmware update now? The device will reboot if the update succeeds.")) {
        return;
    }

    setFormBusy(form, true, "Updating...");
    setFormStatus(form, "busy", "Downloading and flashing firmware...");
    try {
        const formData = new FormData(form);
        const responseData = await postFwForm({url, formData});
        setFormStatus(form, "ok", responseData.message || "Firmware update started");
    } catch (error) {
        console.error(error);
        setFormStatus(form, "error", error.message || "Firmware update failed");
    } finally {
        setFormBusy(form, false);
    }
};

const postSampleAsJson = async ({url, formData}) => {
    const formObj = Object.fromEntries(formData.entries());
    return fetchPayloadOrError(url, {
        method: "POST",
        mode: "cors",
        keepalive: false,
        headers: {"Accept": "application/json", "Content-Type": "application/json"},
        body: JSON.stringify(formObj)
    });
};

const postCfgDataAsJson = async ({url, formData}) => {
    const formObj = Object.fromEntries(formData.entries());
    formObj.port = parseInt(formObj.port, 10);
    formObj.v4mask = parseInt(formObj.v4mask, 10);
    formObj.sensor_retries = parseInt(formObj.sensor_retries, 10);
    formObj.sensor_poll_delay = parseInt(formObj.sensor_poll_delay, 10);
    formObj.wifi_wpa2ent = (formObj.wifi_wpa2ent === "on");
    formObj.v4dhcp = (formObj.v4dhcp === "on");
    formObj.mqtt_enable = (formObj.mqtt_enable === "on");
    formObj.sensor_enable = (formObj.sensor_enable === "on");

    return fetchPayloadOrError(url, {
        method: "POST",
        mode: "cors",
        keepalive: false,
        headers: {"Accept": "application/json", "Content-Type": "application/json"},
        body: JSON.stringify(formObj)
    });
};

const postFwForm = async ({url, formData}) => {
    const params = new URLSearchParams(formData);
    return fetchPayloadOrError(url, {
        method: "POST",
        body: params
    });
};

// EOF
