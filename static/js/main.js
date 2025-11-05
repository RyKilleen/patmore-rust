import { getItems, updateItem } from "./request.js";

let s;
const main = async () => {
  refreshList();
  initWebsockets();
};

const initWebsockets = () => {
  s = new WebSocket(
    (window.location.protocol === "https:" ? "wss://" : "ws://") +
      window.location.host +
      "/ws/updates",
  );

  s.onmessage = function (e) {
    const newItems = JSON.parse(e.data);
    createListFromItems(newItems);
  };
};

const refreshList = async () => {
  const items = await getItems();
  createListFromItems(items);
};

const handleCheck = async (label) => {
  s.send(JSON.stringify({ type: "toggle", label }));
};

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

const createListFromItems = (items) => {
  const listId = "item-list";

  let container =
    document.getElementById(listId) ||
    (() => {
      const el = document.createElement("div");
      el.id = listId;
      el.classList.add("list-container");
      el.addEventListener("click", (e) => {
        if (e.target.type !== "checkbox") return;
        handleCheck(e.target.value);
      });
      document.body.appendChild(el);
      return el;
    })();

  const grouped = groupByAisle(items);

  let htmlString = "";
  for (const [aisle, aisleItems] of Object.entries(grouped)) {
    htmlString += `
      <details class="aisle-group" open>
        <summary class="aisle-title">${aisle}</summary>
        <ul class="aisle-list">
          ${aisleItems.map((x) => createListItemString(x)).join("")}
        </ul>
      </details>
    `;
  }

  container.innerHTML = htmlString;
  return container;
};

const createListItemString = (item) => `
  <li class="item">
    <label class="item-label">
      <input type="checkbox" class="item-checkbox" ${
        item.needed ? "checked" : ""
      } value="${item.label}" />
      <span class="toggle-switch" aria-hidden="true"></span>
      <span class="item-text">${item.label}</span>
    </label>
  </li>
`;

main();
