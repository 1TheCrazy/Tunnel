const invoke = window.__TAURI__.core.invoke;

const elements = {
  addServer: document.querySelector("#add-server"),
  serverName: document.querySelector("#server-name"),
  serverHost: document.querySelector("#server-host"),
  serverPassword: document.querySelector("#server-password"),
  serverFingerprint: document.querySelector("#server-fingerprint"),
  serverSelector: document.querySelector("#server-selector"),
  serverToggle: document.querySelector("#server-toggle"),
  openAddServer: document.querySelector("#open-add-server"),
  closeAddServer: document.querySelector("#close-add-server"),
  addOverlay: document.querySelector("#add-overlay"),
  activeServer: document.querySelector("#active-server"),
  serverTitle: document.querySelector("#server-title"),
  serverList: document.querySelector("#server-list"),
  nodes: document.querySelector("#nodes"),
  listView: document.querySelector("#list-view"),
  mapView: document.querySelector("#map-view"),
  mapSurface: document.querySelector("#map-surface"),
  mapCanvas: document.querySelector("#map-canvas"),
  mapNodes: document.querySelector("#map-nodes"),
  mapEmpty: document.querySelector("#map-empty"),
  viewSwitch: document.querySelector("#view-switch"),
  nodeCount: document.querySelector("#node-count"),
  refresh: document.querySelector("#refresh"),
  disconnect: document.querySelector("#disconnect"),
  networkMonitor: document.querySelector("#network-monitor"),
  networkRateSummary: document.querySelector("#network-rate-summary"),
  networkFullscreen: document.querySelector("#network-fullscreen"),
  networkReceivedTotal: document.querySelector("#network-received-total"),
  networkSentTotal: document.querySelector("#network-sent-total"),
  networkReceivedRate: document.querySelector("#network-received-rate"),
  networkSentRate: document.querySelector("#network-sent-rate"),
  networkReceivedGraph: document.querySelector("#network-received-graph"),
  networkSentGraph: document.querySelector("#network-sent-graph"),
  networkOverlay: document.querySelector("#network-overlay"),
  closeNetworkFullscreen: document.querySelector("#close-network-fullscreen"),
  networkInterface: document.querySelector("#network-interface"),
  networkFullscreenReceivedRate: document.querySelector("#network-fullscreen-received-rate"),
  networkFullscreenSentRate: document.querySelector("#network-fullscreen-sent-rate"),
  networkFullscreenReceivedTotal: document.querySelector("#network-fullscreen-received-total"),
  networkFullscreenSentTotal: document.querySelector("#network-fullscreen-sent-total"),
  networkFullscreenReceivedCaption: document.querySelector("#network-fullscreen-received-caption"),
  networkFullscreenSentCaption: document.querySelector("#network-fullscreen-sent-caption"),
  networkFullscreenReceivedGraph: document.querySelector("#network-fullscreen-received-graph"),
  networkFullscreenSentGraph: document.querySelector("#network-fullscreen-sent-graph"),
  nodesEmpty: document.querySelector("#nodes-empty").parentElement,
  status: document.querySelector("#status"),
};

let statusTimer;
let hasServers = false;
let connectionActive = false;
let activeView = "list";
let latestNodes = [];
let latestHasActiveServer = false;
let networkPollTimer;
let networkSampleError = false;
const networkSamples = [];
const networkSampleLimit = 120;
const mapTransform = { scale: 1, x: 0, y: 0, width: 0, height: 0 };
const mapPointers = new Map();
let dragOrigin;
let pinchOrigin;
const svgNamespace = "http://www.w3.org/2000/svg";
const webMercatorLimit = 85.05112878;
const maximumMapZoom = 24;
const mapCenterLatitude = 40;

function showStatus(message, error = false) {
  window.clearTimeout(statusTimer);
  elements.status.textContent = message;
  elements.status.classList.toggle("error", error);
  elements.status.classList.add("visible");
  statusTimer = window.setTimeout(() => {
    elements.status.classList.remove("visible");
  }, 4200);
}

async function runAction(action, successMessage) {
  setBusy(true);
  try {
    await action();
    await load();
    if (successMessage) {
      showStatus(successMessage);
    }
  } catch (error) {
    showStatus(String(error), true);
  } finally {
    setBusy(false);
  }
}

function setBusy(busy) {
  document.querySelectorAll("button, input").forEach((element) => {
    element.disabled = busy;
  });
  elements.disconnect.disabled = busy || !connectionActive;
}

function openServerList(open) {
  elements.serverSelector.classList.toggle("open", open);
  elements.serverSelector.setAttribute("aria-expanded", String(open));
}

function openAddModal(open) {
  if (!hasServers && !open) {
    return;
  }

  elements.addOverlay.classList.toggle("open", open);
  elements.addOverlay.setAttribute("aria-hidden", String(!open));

  if (open) {
    window.setTimeout(() => elements.serverName.focus(), 0);
  }
}

function iconButton(label, pathMarkup, onClick, className = "") {
  const element = document.createElement("button");
  element.type = "button";
  element.className = className ? `icon-button ${className}` : "icon-button";
  element.title = label;
  element.setAttribute("aria-label", label);
  element.innerHTML = `<svg viewBox="0 0 24 24" aria-hidden="true">${pathMarkup}</svg>`;
  element.addEventListener("click", onClick);
  return element;
}

function button(label, onClick, className = "") {
  const element = document.createElement("button");
  element.type = "button";
  element.textContent = label;
  element.className = className;
  element.addEventListener("click", onClick);
  return element;
}

function renderServers(servers) {
  hasServers = servers.length > 0;
  document.body.classList.toggle("no-servers", !hasServers);
  elements.serverSelector.hidden = !hasServers;
  elements.serverList.replaceChildren();

  const activeServer = servers.find((server) => server.active);
  elements.activeServer.textContent = activeServer ? activeServer.name : "None";
  elements.serverTitle.textContent = activeServer ? activeServer.name : "No active server";

  if (!hasServers) {
    openServerList(false);
    openAddModal(true);
    return;
  }

  for (const server of servers) {
    const option = document.createElement("div");
    option.className = "server-option";
    option.classList.toggle("active", server.active);

    const select = document.createElement("button");
    select.type = "button";
    select.className = "server-option-select";
    select.innerHTML = `
      <span class="server-option-title"></span>
      <span class="server-option-meta"></span>
    `;
    select.querySelector(".server-option-title").textContent = server.name;
    select.querySelector(".server-option-meta").textContent = `${server.host} - ${server.nodes} ${server.nodes === 1 ? "node" : "nodes"}`;
    select.addEventListener("click", () => {
      runAction(
        () => invoke("server_set", { name: server.name }),
        `'${server.name}' is now the active server`,
      ).then(() => openServerList(false));
    });

    const remove = iconButton(
      `Remove ${server.name}`,
      '<path d="M3 6h18" /><path d="M8 6V4h8v2" /><path d="m19 6-1 14H6L5 6" /><path d="M10 11v5" /><path d="M14 11v5" />',
      () => runAction(
        () => invoke("server_remove", { name: server.name }),
        `Successfully removed the server '${server.name}'`,
      ),
      "remove-server",
    );

    option.append(select, remove);
    elements.serverList.append(option);
  }
}

function renderConnection(connected) {
  connectionActive = connected;
  elements.disconnect.disabled = !connected;
  void invoke("set_connection_icon", { connected }).catch(() => {});
  syncNetworkPolling(connected);
}

function syncNetworkPolling(connected) {
  if (!connected) {
    connectionActive = false;
    window.clearInterval(networkPollTimer);
    networkPollTimer = undefined;
    networkSamples.length = 0;
    elements.networkMonitor.hidden = true;
    elements.disconnect.disabled = true;
    openNetworkMonitor(false);
    openNetworkFullscreen(false);
    return;
  }

  if (!networkPollTimer) {
    pollNetworkStats();
    networkPollTimer = window.setInterval(pollNetworkStats, 1000);
  }
}

async function pollNetworkStats() {
  try {
    const stats = await invoke("network_stats");
    if (!stats) {
      syncNetworkPolling(false);
      return;
    }

    networkSampleError = false;
    updateNetworkStats(stats);
  } catch (error) {
    if (!networkSampleError) {
      showStatus(`Couldn't retrieve network activity: ${String(error)}`, true);
      networkSampleError = true;
    }
  }
}

function updateNetworkStats(stats) {
  const now = Date.now();
  const previous = networkSamples.at(-1);
  const elapsedSeconds = previous ? Math.max((now - previous.time) / 1000, 0.001) : 1;
  const receivedRate = previous && stats.received_bytes >= previous.receivedBytes
    ? (stats.received_bytes - previous.receivedBytes) / elapsedSeconds
    : 0;
  const sentRate = previous && stats.sent_bytes >= previous.sentBytes
    ? (stats.sent_bytes - previous.sentBytes) / elapsedSeconds
    : 0;
  const sample = {
    time: now,
    interface: stats.interface,
    receivedBytes: stats.received_bytes,
    sentBytes: stats.sent_bytes,
    receivedRate,
    sentRate,
  };

  networkSamples.push(sample);
  if (networkSamples.length > networkSampleLimit) {
    networkSamples.shift();
  }

  elements.networkMonitor.hidden = false;
  elements.networkRateSummary.textContent = `Down ${formatRate(receivedRate)} | Up ${formatRate(sentRate)}`;
  elements.networkReceivedTotal.textContent = formatBytes(stats.received_bytes);
  elements.networkSentTotal.textContent = formatBytes(stats.sent_bytes);
  elements.networkReceivedRate.textContent = formatRate(receivedRate);
  elements.networkSentRate.textContent = formatRate(sentRate);
  elements.networkInterface.textContent = stats.interface;
  elements.networkFullscreenReceivedRate.textContent = formatRate(receivedRate);
  elements.networkFullscreenSentRate.textContent = formatRate(sentRate);
  elements.networkFullscreenReceivedTotal.textContent = formatBytes(stats.received_bytes);
  elements.networkFullscreenSentTotal.textContent = formatBytes(stats.sent_bytes);
  elements.networkFullscreenReceivedCaption.textContent = formatRate(receivedRate);
  elements.networkFullscreenSentCaption.textContent = formatRate(sentRate);
  updateNetworkGraph(elements.networkReceivedGraph, networkSamples.map((entry) => entry.receivedRate));
  updateNetworkGraph(elements.networkSentGraph, networkSamples.map((entry) => entry.sentRate));
  updateNetworkGraph(elements.networkFullscreenReceivedGraph, networkSamples.map((entry) => entry.receivedRate));
  updateNetworkGraph(elements.networkFullscreenSentGraph, networkSamples.map((entry) => entry.sentRate));
}

function formatBytes(bytes) {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = bytes;
  let unit = 0;
  while (value >= 1000 && unit < units.length - 1) {
    value /= 1000;
    unit += 1;
  }
  return `${value >= 100 || unit === 0 ? value.toFixed(0) : value.toFixed(1)} ${units[unit]}`;
}

function formatRate(bytesPerSecond) {
  return `${formatBytes(bytesPerSecond)}/s`;
}

function updateNetworkGraph(graph, values) {
  const maximum = Math.max(...values, 1);
  const lastIndex = Math.max(values.length - 1, 1);
  graph.setAttribute("points", values
    .map((value, index) => `${(index / lastIndex) * 100},${100 - ((value / maximum) * 92) - 4}`)
    .join(" "));
}

function openNetworkMonitor(open) {
  elements.networkMonitor.classList.toggle("open", open);
  elements.networkMonitor.setAttribute("aria-expanded", String(open));
}

function openNetworkFullscreen(open) {
  elements.networkOverlay.classList.toggle("open", open);
  elements.networkOverlay.setAttribute("aria-hidden", String(!open));
}

function renderNodes(nodes, hasActiveServer) {
  latestNodes = nodes;
  latestHasActiveServer = hasActiveServer;
  elements.nodes.replaceChildren();
  elements.nodeCount.textContent = hasActiveServer
    ? `${nodes.length} ${nodes.length === 1 ? "node" : "nodes"}`
    : "No active server";
  elements.nodesEmpty.classList.toggle("is-empty", nodes.length === 0);

  for (const node of nodes) {
    const row = document.createElement("tr");
    row.append(cell(node.name || "Unnamed node"), cell(node.id), cell(node.ip), cell(String(node.discovered)));

    const actions = document.createElement("td");
    const wrap = document.createElement("div");
    wrap.className = "actions";
    const connectButton = button(
      node.connected ? "Connected" : "Connect",
      () => runAction(() => invoke("connect", { id: node.id }), `Connected to node '${node.id}'`),
      node.connected ? "connected-node" : "",
    );
    connectButton.disabled = node.connected;
    wrap.append(connectButton);
    actions.append(wrap);
    row.append(actions);
    elements.nodes.append(row);
  }

  renderMapNodes(nodes, hasActiveServer);
}

function renderMapNodes(nodes, hasActiveServer) {
  elements.mapNodes.replaceChildren();
  const mappedNodes = nodes.filter((node) => Number.isFinite(node.latitude) && Number.isFinite(node.longitude));
  elements.mapEmpty.hidden = mappedNodes.length > 0;
  elements.mapEmpty.textContent = hasActiveServer
    ? "No mapped nodes are available"
    : "Select a server to view its nodes";

  for (const group of buildMapNodeGroups(mappedNodes)) {
    const wrapper = document.createElement("div");
    wrapper.className = "map-node-group";
    wrapper.classList.toggle("clustered", group.members.length > 1);
    wrapper.dataset.mapX = group.x;
    wrapper.dataset.mapY = group.y;
    wrapper.dataset.worldOffset = group.worldOffset;

    if (group.members.length > 1) {
      const count = document.createElement("span");
      count.className = "map-node-cluster-count";
      count.textContent = group.members.length;
      wrapper.append(count);
    }

    group.members.forEach(({ node }, index) => {
      const marker = document.createElement("button");
      marker.type = "button";
      marker.className = "map-node";
      marker.title = `${node.name || node.id} - ${node.ip}`;
      marker.dataset.latitude = node.latitude;
      marker.dataset.longitude = node.longitude;
      marker.classList.toggle("connected", node.connected);
      marker.setAttribute("aria-label", `Connect to node ${node.name || node.id} at ${node.ip}`);
      marker.tabIndex = group.worldOffset === 0 ? 0 : -1;
      marker.setAttribute("aria-hidden", String(group.worldOffset !== 0));
      if (group.members.length > 1) {
        const angle = ((Math.PI * 2 * index) / group.members.length) - (Math.PI / 2);
        const radius = Math.max(30, Math.min(48, 20 + (group.members.length * 3)));
        marker.style.setProperty("--explode-x", `${Math.cos(angle) * radius}px`);
        marker.style.setProperty("--explode-y", `${Math.sin(angle) * radius}px`);
      }
      const tooltip = document.createElement("span");
      tooltip.className = "map-node-tooltip";
      tooltip.innerHTML = `
        <strong></strong>
        <span class="map-node-ip"></span>
        <span class="map-node-status"></span>
      `;
      tooltip.querySelector("strong").textContent = node.name || node.id;
      tooltip.querySelector(".map-node-ip").textContent = node.ip;
      tooltip.querySelector(".map-node-status").textContent = node.connected
        ? "Connected"
        : node.discovered ? "Discovered" : "Not discovered";
      marker.append(tooltip);
      if (!node.connected) {
        marker.addEventListener("click", () => {
          runAction(() => invoke("connect", { id: node.id }), `Connected to node '${node.id}'`);
        });
      }
      wrapper.append(marker);
    });

    elements.mapNodes.append(wrapper);
  }

  positionMapNodes();
}

function buildMapNodeGroups(nodes) {
  const mapSize = Math.max(mapTransform.width * mapTransform.scale, window.innerWidth, window.innerHeight);
  const proximity = 28 / mapSize;
  const groups = [];

  for (const worldOffset of [-1, 0, 1]) {
    for (const node of nodes) {
      const point = projectWebMercator(node.latitude, node.longitude);
      const group = groups.find((candidate) => candidate.worldOffset === worldOffset
        && Math.hypot(point.x - candidate.x, point.y - candidate.y) <= proximity);

      if (group) {
        group.members.push({ node, point });
        group.x = group.members.reduce((total, member) => total + member.point.x, 0) / group.members.length;
        group.y = group.members.reduce((total, member) => total + member.point.y, 0) / group.members.length;
      } else {
        groups.push({ x: point.x, y: point.y, worldOffset, members: [{ node, point }] });
      }
    }
  }

  return groups;
}

function setView(view) {
  activeView = view;
  elements.listView.hidden = view !== "list";
  elements.mapView.hidden = view !== "map";
  document.querySelector(".app").classList.toggle("map-active", view === "map");
  elements.viewSwitch.dataset.active = view;

  elements.viewSwitch.querySelectorAll(".mode-option").forEach((option) => {
    const selected = option.dataset.view === view;
    option.classList.toggle("active", selected);
    option.setAttribute("aria-pressed", String(selected));
  });

  if (view === "map") {
    window.requestAnimationFrame(updateMapCanvas);
  }
}

function updateMapCanvas() {
  const bounds = elements.mapSurface.getBoundingClientRect();
  const side = Math.max(bounds.width, bounds.height);
  mapTransform.width = side;
  mapTransform.height = side;
  applyMapTransform();
  renderMapNodes(latestNodes, latestHasActiveServer);
}

function applyMapTransform() {
  const scaledWidth = mapTransform.width * mapTransform.scale;
  const scaledHeight = mapTransform.height * mapTransform.scale;

  normalizeMapX(scaledWidth);
  constrainMapY();
  const visualY = mapBaseYOffset(scaledHeight) + mapTransform.y;
  elements.mapCanvas.style.width = `${scaledWidth * 3}px`;
  elements.mapCanvas.style.height = `${scaledHeight}px`;
  elements.mapCanvas.style.left = `calc(50% + ${mapTransform.x}px)`;
  elements.mapCanvas.style.top = `calc(50% + ${visualY}px)`;
  elements.mapCanvas.style.transform = "translate(-50%, -50%)";
  positionMapNodes();
}

function mapBaseYOffset(scaledHeight) {
  const centerPoint = projectWebMercator(mapCenterLatitude, 0);
  return (0.5 - centerPoint.y) * scaledHeight;
}

function normalizeMapX(worldWidth) {
  if (worldWidth === 0) {
    return;
  }

  mapTransform.x = ((mapTransform.x + (worldWidth / 2)) % worldWidth + worldWidth) % worldWidth - (worldWidth / 2);
}

function constrainMapY() {
  const maximumOffset = (mapTransform.height * (mapTransform.scale - 1)) / 2;
  mapTransform.y = Math.min(Math.max(mapTransform.y, -maximumOffset), maximumOffset);
}

function projectWebMercator(latitude, longitude) {
  const clampedLatitude = Math.min(Math.max(latitude, -webMercatorLimit), webMercatorLimit);
  const latitudeRadians = clampedLatitude * (Math.PI / 180);

  return {
    x: (longitude + 180) / 360,
    y: (1 - (Math.log(Math.tan((Math.PI / 4) + (latitudeRadians / 2))) / Math.PI)) / 2,
  };
}

function positionMapNodes() {
  const bounds = elements.mapSurface.getBoundingClientRect();
  const scaledWidth = mapTransform.width * mapTransform.scale;
  const scaledHeight = mapTransform.height * mapTransform.scale;
  const left = (bounds.width / 2) + mapTransform.x - (scaledWidth / 2);
  const top = (bounds.height / 2) + mapBaseYOffset(scaledHeight) + mapTransform.y - (scaledHeight / 2);

  elements.mapNodes.querySelectorAll(".map-node-group").forEach((group) => {
    const mapX = Number(group.dataset.mapX);
    const mapY = Number(group.dataset.mapY);
    const worldOffset = Number(group.dataset.worldOffset);
    group.style.left = `${left + ((mapX + worldOffset) * scaledWidth)}px`;
    group.style.top = `${top + (mapY * scaledHeight)}px`;
  });
}

function appendRingPath(path, ring) {
  let previousX;

  ring.forEach(([longitude, latitude], index) => {
    const point = projectWebMercator(latitude, longitude);
    const crossesDateLine = previousX !== undefined && Math.abs(point.x - previousX) > 0.5;
    path.push(index === 0 || crossesDateLine ? "M" : "L", point.x * 1024, point.y * 1024);
    previousX = point.x;
  });

  path.push("Z");
}

function countryPath(geometry) {
  const polygons = geometry.type === "Polygon" ? [geometry.coordinates] : geometry.coordinates;
  const path = [];

  polygons.forEach((polygon) => polygon.forEach((ring) => appendRingPath(path, ring)));
  return path.join(" ");
}

async function loadWorldMap() {
  const response = await fetch("assets/world-countries.geojson");
  if (!response.ok) {
    throw new Error("Couldn't load the world map");
  }

  const world = await response.json();
  const mapLayer = document.createElementNS(svgNamespace, "g");
  mapLayer.id = "country-shapes";

  world.features
    .filter((feature) => feature.properties.ISO_A2 !== "AQ" && feature.properties.CONTINENT !== "Antarctica")
    .forEach((feature) => {
      const path = document.createElementNS(svgNamespace, "path");
      path.setAttribute("d", countryPath(feature.geometry));
      path.classList.add("country-shape");
      mapLayer.append(path);
    });

  const leftWorld = document.createElementNS(svgNamespace, "use");
  leftWorld.setAttribute("href", "#country-shapes");
  leftWorld.setAttribute("x", "-1024");

  const rightWorld = document.createElementNS(svgNamespace, "use");
  rightWorld.setAttribute("href", "#country-shapes");
  rightWorld.setAttribute("x", "1024");

  elements.mapCanvas.replaceChildren(leftWorld, mapLayer, rightWorld);
}

function zoomMap(nextScale, clientX, clientY) {
  const scale = Math.min(Math.max(nextScale, 1), maximumMapZoom);
  const bounds = elements.mapSurface.getBoundingClientRect();
  const pointerX = clientX - bounds.left - (bounds.width / 2);
  const pointerY = clientY - bounds.top - (bounds.height / 2);
  const scaleRatio = scale / mapTransform.scale;
  const currentBaseY = mapBaseYOffset(mapTransform.height * mapTransform.scale);
  const nextBaseY = mapBaseYOffset(mapTransform.height * scale);

  mapTransform.x = pointerX - ((pointerX - mapTransform.x) * scaleRatio);
  const visualY = currentBaseY + mapTransform.y;
  mapTransform.y = pointerY - ((pointerY - visualY) * scaleRatio) - nextBaseY;
  mapTransform.scale = scale;
  renderMapNodes(latestNodes, latestHasActiveServer);
  applyMapTransform();
}

function pointerDistance(first, second) {
  return Math.hypot(first.clientX - second.clientX, first.clientY - second.clientY);
}

function pointerMidpoint(first, second) {
  return {
    x: (first.clientX + second.clientX) / 2,
    y: (first.clientY + second.clientY) / 2,
  };
}

function cell(text) {
  const element = document.createElement("td");
  element.textContent = text;
  element.title = text;
  return element;
}

async function load() {
  const [servers, connected] = await Promise.all([
    invoke("list_servers"),
    invoke("connection_active"),
  ]);
  renderServers(servers);
  renderConnection(connected);

  try {
    const nodes = await invoke("list_nodes");
    renderNodes(nodes, true);
  } catch {
    renderNodes([], false);
  }
}

elements.serverSelector.addEventListener("click", (event) => {
  if (event.target.closest("#open-add-server")) {
    return;
  }

  if (event.target.closest(".server-list")) {
    return;
  }

  openServerList(!elements.serverSelector.classList.contains("open"));
});

elements.serverSelector.addEventListener("keydown", (event) => {
  if (event.key === "Enter" || event.key === " ") {
    event.preventDefault();
    openServerList(!elements.serverSelector.classList.contains("open"));
  }
});

elements.networkMonitor.addEventListener("click", (event) => {
  if (event.target.closest("#network-fullscreen")) {
    return;
  }

  openNetworkMonitor(!elements.networkMonitor.classList.contains("open"));
});

elements.networkMonitor.addEventListener("keydown", (event) => {
  if (event.key === "Enter" || event.key === " ") {
    event.preventDefault();
    openNetworkMonitor(!elements.networkMonitor.classList.contains("open"));
  }
});

elements.networkFullscreen.addEventListener("click", (event) => {
  event.stopPropagation();
  openNetworkFullscreen(true);
});

elements.closeNetworkFullscreen.addEventListener("click", () => {
  openNetworkFullscreen(false);
});

elements.networkOverlay.addEventListener("click", (event) => {
  if (event.target === elements.networkOverlay) {
    openNetworkFullscreen(false);
  }
});

elements.openAddServer.addEventListener("click", (event) => {
  event.stopPropagation();
  openServerList(true);
  openAddModal(true);
});

elements.closeAddServer.addEventListener("click", () => {
  openAddModal(false);
});

elements.addOverlay.addEventListener("click", (event) => {
  if (event.target === elements.addOverlay) {
    openAddModal(false);
  }
});

elements.addServer.addEventListener("submit", (event) => {
  event.preventDefault();
  const name = elements.serverName.value.trim();
  const host = elements.serverHost.value.trim();
  const password = elements.serverPassword.value;
  const fingerprint = elements.serverFingerprint.value.trim();

  runAction(
    () => invoke("server_add", { name, host, password: password || null, fingerprint: fingerprint || null }),
    "Successfully added the server",
  ).then(() => {
    elements.addServer.reset();
    openAddModal(false);
  });
});

elements.refresh.addEventListener("click", () => {
  runAction(async () => {
    await invoke("refresh");
    await invoke("refresh_node_locations");
  }, "Refreshing nodes and locations complete");
});

elements.disconnect.addEventListener("click", () => {
  runAction(() => invoke("disconnect"), "Disconnected");
});

elements.viewSwitch.addEventListener("click", (event) => {
  const option = event.target.closest(".mode-option");
  if (option) {
    setView(option.dataset.view);
  }
});

elements.mapSurface.addEventListener("wheel", (event) => {
  event.preventDefault();
  zoomMap(mapTransform.scale * (event.deltaY < 0 ? 1.12 : 0.89), event.clientX, event.clientY);
}, { passive: false });

elements.mapSurface.addEventListener("pointerdown", (event) => {
  if (event.target.closest(".map-node-group")) {
    return;
  }

  elements.mapSurface.setPointerCapture(event.pointerId);
  mapPointers.set(event.pointerId, event);

  if (mapPointers.size === 1) {
    dragOrigin = { x: event.clientX, y: event.clientY, mapX: mapTransform.x, mapY: mapTransform.y };
  } else if (mapPointers.size === 2) {
    const [first, second] = mapPointers.values();
    pinchOrigin = { distance: pointerDistance(first, second), scale: mapTransform.scale };
  }
});

elements.mapSurface.addEventListener("pointermove", (event) => {
  if (!mapPointers.has(event.pointerId)) {
    return;
  }

  mapPointers.set(event.pointerId, event);

  if (mapPointers.size === 1 && dragOrigin) {
    mapTransform.x = dragOrigin.mapX + event.clientX - dragOrigin.x;
    mapTransform.y = dragOrigin.mapY + event.clientY - dragOrigin.y;
    applyMapTransform();
  } else if (mapPointers.size === 2 && pinchOrigin) {
    const [first, second] = mapPointers.values();
    const midpoint = pointerMidpoint(first, second);
    zoomMap(pinchOrigin.scale * (pointerDistance(first, second) / pinchOrigin.distance), midpoint.x, midpoint.y);
  }
});

function endMapPointer(event) {
  mapPointers.delete(event.pointerId);
  if (mapPointers.size < 2) {
    pinchOrigin = undefined;
  }
  if (mapPointers.size === 0) {
    dragOrigin = undefined;
  }
}

elements.mapSurface.addEventListener("pointerup", endMapPointer);
elements.mapSurface.addEventListener("pointercancel", endMapPointer);
window.addEventListener("resize", () => {
  if (activeView === "map") {
    updateMapCanvas();
  }
});

loadWorldMap().catch((error) => showStatus(String(error), true));

load()
  .then(async () => {
    try {
      await invoke("refresh_node_locations");
      await load();
    } catch (error) {
      showStatus(`Couldn't refresh node locations: ${String(error)}`, true);
    }
  })
  .catch((error) => showStatus(String(error), true));
