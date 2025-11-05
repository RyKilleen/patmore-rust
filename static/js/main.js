import { getItems } from "./request.js";

let s;
let allItems = [];
let showNeededOnly = false;

const main = async () => {
  await refreshList();
  initWebsockets();
  initFilterToggle();
};

const initWebsockets = () => {
  s = new WebSocket(
    (window.location.protocol === "https:" ? "wss://" : "ws://") +
      window.location.host +
      "/ws/updates",
  );

  s.onmessage = (e) => {
    allItems = JSON.parse(e.data);
    renderList();
  };
};

const refreshList = async () => {
  allItems = await getItems();
  renderList();
};

const handleCheck = (label, checked) => {
  s.send(JSON.stringify({ type: "toggle", label }));
  // Update the local state immediately
  const item = allItems.find((i) => i.label === label);
  if (item) item.needed = checked;
};

// Render filtered list with in-place DOM updates
const renderList = () => {
  const itemsToShow = showNeededOnly
    ? allItems.filter((item) => item.needed)
    : allItems;

  const container =
    document.getElementById("item-list") || createListContainer();
  const grouped = groupByAisle(itemsToShow);

  // Keep track of current open aisles
  const openAisles = {};
  container.querySelectorAll(".aisle-group").forEach((d) => {
    const title = d.dataset.aisle;
    openAisles[title] = d.open;
  });

  // Update or create sections
  for (const [aisle, items] of Object.entries(grouped)) {
    let details = container.querySelector(
      `.aisle-group[data-aisle="${aisle}"]`,
    );
    if (!details) {
      // Create new section
      details = document.createElement("details");
      details.className = "aisle-group";
      details.dataset.aisle = aisle;

      const summary = document.createElement("summary");
      summary.className = "aisle-title";
      summary.textContent = aisle;
      details.appendChild(summary);

      const ul = document.createElement("ul");
      ul.className = "aisle-list";
      details.appendChild(ul);

      container.appendChild(details);
    }

    // Restore open state
    if (openAisles[aisle] !== undefined) {
      details.open = openAisles[aisle];
    } else {
      details.open = true;
    }

    const ul = details.querySelector(".aisle-list");

    // Update existing items or add new ones
    const existingItems = {};
    ul.querySelectorAll(".item").forEach((li) => {
      const label = li.dataset.label;
      existingItems[label] = li;
    });

    items.forEach((item) => {
      let li = existingItems[item.label];
      if (!li) {
        // Create new list item
        li = createListItemElement(item);
        ul.appendChild(li);
      } else {
        // Update checkbox state
        const checkbox = li.querySelector("input[type=checkbox]");
        checkbox.checked = item.needed;
      }
    });

    // Remove items that no longer exist
    ul.querySelectorAll(".item").forEach((li) => {
      if (!items.find((i) => i.label === li.dataset.label)) {
        ul.removeChild(li);
      }
    });
  }

  // Remove aisles that no longer exist
  container.querySelectorAll(".aisle-group").forEach((details) => {
    const aisle = details.dataset.aisle;
    if (!grouped[aisle]) container.removeChild(details);
  });
};

const createListContainer = () => {
  const el = document.createElement("div");
  el.id = "item-list";
  el.className = "list-container";
  document.body.appendChild(el);
  return el;
};

const createListItemElement = (item) => {
  const li = document.createElement("li");
  li.className = "item";
  li.dataset.label = item.label;

  const label = document.createElement("label");
  label.className = "toggle-switch-wrapper";

  const checkbox = document.createElement("input");
  checkbox.type = "checkbox";
  checkbox.className = "item-checkbox";
  checkbox.checked = item.needed;
  checkbox.addEventListener("change", (e) =>
    handleCheck(item.label, e.target.checked),
  );

  const toggle = document.createElement("span");
  toggle.className = "toggle-switch";
  toggle.setAttribute("aria-hidden", "true");

  const text = document.createElement("span");
  text.className = "item-text";
  text.textContent = item.label;

  label.appendChild(checkbox);
  label.appendChild(toggle);
  label.appendChild(text);
  li.appendChild(label);

  return li;
};

// Group by aisle
const groupByAisle = (items) => {
  const grouped = {};
  items.forEach((item) => {
    item.aisle.forEach((aisle) => {
      if (!grouped[aisle]) grouped[aisle] = [];
      grouped[aisle].push(item);
    });
  });
  return Object.fromEntries(
    Object.entries(grouped).sort(([a], [b]) => a.localeCompare(b)),
  );
};

// Initialize the "Show only needed" filter
const initFilterToggle = () => {
  const container = document.createElement("div");
  container.className = "filter-toggle-container";

  const label = document.createElement("label");
  label.className = "toggle-switch-wrapper";

  const checkbox = document.createElement("input");
  checkbox.type = "checkbox";
  checkbox.id = "filter-toggle";
  checkbox.addEventListener("change", (e) => {
    showNeededOnly = e.target.checked;
    renderList();
  });

  const toggle = document.createElement("span");
  toggle.className = "toggle-switch";
  toggle.setAttribute("aria-hidden", "true");

  const text = document.createElement("span");
  text.className = "toggle-label-text";
  text.textContent = "Shopping Mode";

  label.appendChild(checkbox);
  label.appendChild(toggle);
  label.appendChild(text);
  container.appendChild(label);

  document.body.prepend(container);
};

main();
